use crate::prelude::*;



pub fn parse_tua_file<'a> (file: &'a LexedTuaFile, path: &Path, logger: &mut Logger) -> (ParsedTuaFile<'a>, Vec<CompileError>) {
	let contents = &file.contents;
	let mut definitions = vec!();
	let mut errors = vec!();
	let mut index = 0;
	while index < contents.len() {
		match &contents[index].token {
			RawToken::Name(name) => {
				let new_definition = match parse_definition(name, &mut index, contents, &mut errors, logger) {
					Ok(v) => v,
					Err(error) => {
						errors.push(error);
						return (ParsedTuaFile {definitions}, errors);
					},
				};
				definitions.push(new_definition);
			}
			_ => {
				errors.push(RawCompileError::new_unexpected_token(&contents[index], "'function', 'object', 'choice', 'type', 'static', 'use', or '#'", "while parsing top-level definitions (detecting token type)").into());
				return (ParsedTuaFile {definitions}, errors);
			}
		}
	}
	(ParsedTuaFile {definitions}, errors)
}



pub fn parse_definition<'a> (name: &str, index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTDefinition<'a>, CompileError> {
	match name {
		"function" => parse_function_definition(index, contents, errors, logger),
		"object"   => parse_object_definition(index, contents, errors, logger),
		"choice"   => parse_choice_definition(index, contents, errors, logger),
		"type"     => parse_type_definition(index, contents, errors, logger),
		"const"    => parse_const_definition(index, contents, errors, logger),
		"static"   => parse_static_definition(index, contents, errors, logger),
		"use"      => {logger.print_all(); todo!("use definition")}
		"#"        => {logger.print_all(); todo!("# definition")}
		_ => Err(RawCompileError::new_unexpected_token(&contents[*index], "'function', 'object', 'choice', 'type', 'static', 'use', or '#'", "while parsing top-level definitions (detecting token content)").into())
	}
}



pub fn get_next_token<'a> (index: &usize, contents: &'a [Token], expected: &str) -> Result<&'a RawToken, CompileError> {
	contents.get(*index)
		.map(|token_data| &token_data.token)
		.ok_or_else(|| RawCompileError::new_unexpected_end_of_file(&contents[*index - 1], expected).into())
}



pub fn get_next_token_checked<'a> (index: &usize, contents: &'a [Token]) -> Option<&'a RawToken> {
	contents.get(*index)
		.map(|token_data| &token_data.token)
}





pub fn parse_type<'a> (index: &mut usize, contents: &'a [Token], _errors: &mut Vec<CompileError>, _logger: &mut Logger) -> Result<ASTType<'a>, CompileError> {

	// type name
	let name_token = get_next_token(index, contents, "[name of type]")?;
	let RawToken::Name(type_name) = name_token else {
		return Err(RawCompileError::InvalidTypeName {location: contents[*index].clone()}.into());
	};
	*index += 1;

	// args
	let mut unnamed_type_arg = None;
	let mut named_type_args = vec!();
	if get_next_token_checked(index, contents) == Some(&RawToken::Operator(Operator::LessThan)) {
		(unnamed_type_arg, named_type_args) = parse_type_args(index, contents, _errors, _logger)?;
	}
	let unnamed_type_arg = unnamed_type_arg.map(|v| box v);

	let mut output = ASTType {
		name: type_name,
		unnamed_type_arg,
		named_type_args,
	};

	// post-fix types
	let output = loop {
		match get_next_token_checked(index, contents) {
			Some(&RawToken::OpenSquareBracket) => {
				*index += 1;
				let array_end_token = get_next_token(index, contents, "]")?;
				if *array_end_token != RawToken::CloseSquareBracket {return Err(RawCompileError::new_unexpected_token(&contents[*index], "']'", "while parsing type array definition").into());}
				output = ASTType {
					name: "Array",
					unnamed_type_arg: Some(box output),
					named_type_args: vec!(),
				};
				*index += 1;
			}
			/*
			Some(&Token::QuestionMark) => {
				output = ASTType {
					name: "Optional",
					type_args: vec!(output),
				};
				*index += 1;
			}
			*/
			_ => break output,
		}
	};

	Ok(output)
}



pub fn parse_type_args<'a> (index: &mut usize, contents: &'a [Token], _errors: &mut Vec<CompileError>, _logger: &mut Logger) -> Result<(Option<ASTType<'a>>, Vec<(&'a str, ASTType<'a>)>), CompileError> {

	let mut unnamed_type_arg = None;
	let mut named_type_args = vec!();
	
	// less than
	if *get_next_token(index, contents, "'<'")? != RawToken::Operator(Operator::LessThan) {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "'<'", "while parsing generics start").into());
	}
	*index += 1;
	let mut i = 0;
	loop {

		// end?
		if *get_next_token(index, contents, "[type], [name], or '>'")? == RawToken::Operator(Operator::GreaterThan) {
			*index += 1;
			break;
		}

		// type arg
		if *get_next_token(&(*index + 1), contents, "[type arg]")? != RawToken::Colon {
			// unnamed type arg
			if i != 0 {
				return Err(RawCompileError::MultipleUnnamedTypes {location: contents[*index].clone()}.into());
			}
			unnamed_type_arg = Some(parse_type(index, contents, _errors, _logger)?);
		} else {
			// named type arg
			let type_name = get_next_token(index, contents, "[name of type]")?;
			let RawToken::Name(type_name) = type_name else {
				return Err(RawCompileError::new_unexpected_token(&contents[*index], "[name of type]", "while parsing generics type name").into());
			};
			*index += 2; // skip colon token
			let ast_type = parse_type(index, contents, _errors, _logger)?;
			named_type_args.push((&**type_name, ast_type));
		}

		match *get_next_token(index, contents, "',' or '>'")? {
			RawToken::Comma => {
				*index += 1;
				i += 1;
				continue;
			}
			RawToken::Operator(Operator::GreaterThan) => {
				*index += 1;
				break;
			},
			_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "',' or '>'", "while parsing generics seperator").into()),
		}

	}

	Ok((unnamed_type_arg, named_type_args))
}





pub fn parse_formula<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	logger.logln(format!("parsing formula at {:?}", contents[*index]));

	let first_item = parse_formula_item(index, contents, errors, logger)?;
	let mut formula_items = vec!(first_item);
	let mut formula_operators = vec!();

	'extra_items: loop {
		let operator = match get_next_token_checked(index, contents) {
			Some(RawToken::Operator(v)) => v,
			_ => break 'extra_items,
		};
		*index += 1;
		let next_item = parse_formula_item(index, contents, errors, logger)?;
		formula_operators.push((operator, operator.get_eval_level()));
		formula_items.push(next_item);
	}

	'combining: for level in (0..=Operator::MAX_EVAL_LEVEL).rev() {
		let mut i = 0;
		'current_level_loop: loop {
			if i >= formula_operators.len() {break 'current_level_loop;}
			let (_, current_eval_level) = formula_operators[i];
			if current_eval_level != level {
				i += 1;
				continue;
			}
			let left = formula_items.remove(i);
			let right = formula_items.remove(i);
			let (current_operator, _) = formula_operators.remove(i);
			formula_items.insert(i, ASTFormula::Operation {left: box left, right: box right, operator: current_operator.clone()});
			// no increment
		}
	}

	Ok(formula_items.pop().unwrap())

}



pub fn parse_formula_item<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {

	// main item / pre-fix operators
	let main_token = get_next_token(index, contents, "[formula item]")?;
	let mut output = match main_token {
		RawToken::Name(value) => {
			match &**value {
				"new" => parse_formula_item_new(index, contents, errors, logger)?,
				_ => {
					*index += 1;
					ASTFormula::Name(value)
				}
			}
		}
		RawToken::Operator(Operator::Not) => parse_formula_item_not(index, contents, errors, logger)?,
		RawToken::OpenParen => parse_formula_item_parens(index, contents, errors, logger)?,
		RawToken::Int(value) => {
			*index += 1;
			ASTFormula::Int(*value)
		}
		RawToken::UInt(value) => {
			*index += 1;
			ASTFormula::UInt(*value)
		}
		RawToken::Float(value) => {
			*index += 1;
			ASTFormula::Float(*value)
		}
		RawToken::Bool(value) => {
			*index += 1;
			ASTFormula::Bool(*value)
		}
		RawToken::String(value) => {
			*index += 1;
			ASTFormula::String(value)
		}
		RawToken::FormattedString {start, items} => parse_formula_item_formatted_string(start, items, index, errors, logger)?,
		RawToken::Char(value) => {
			*index += 1;
			ASTFormula::Char(*value)
		}
		_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "[formula item]", "while parsing next formula item").into()),
	};

	// post-fix operators
	loop {
		let next_token = match get_next_token_checked(index, contents) {
			Some(v) => v,
			None => return Ok(output),
		};
		match *next_token {
			RawToken::OpenSquareBracket      => output = parse_formula_item_index_query(output, index, contents, errors, logger)?,
			RawToken::Period                 => output = parse_formula_item_property_query(output, index, contents, errors, logger)?,
			RawToken::QuestionMark           => output = parse_formula_item_return_test(output, index, contents, errors, logger)?,
			RawToken::OpenParen              => output = parse_formula_item_function_call(output, false, index, contents, errors, logger)?,
			RawToken::Colon                  => output = parse_formula_item_function_call(output, true, index, contents, errors, logger)?,
			RawToken::Operator(Operator::As) => output = parse_formula_item_as(output, index, contents, errors, logger)?,
			_ => return Ok(output),
		}
	}

}



pub fn parse_formula_item_new<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	*index += 1;

	// name
	let object_name = get_next_token(index, contents, "[name of object]")?;
	let RawToken::Name(object_name) = object_name else {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "[name of object]", "while parsing object creation name").into());
	};
	*index += 1;

	// open paren
	let open_paren = get_next_token(index, contents, "'('")?;
	if *open_paren != RawToken::OpenParen {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "'('", "while parsing new object definition").into());
	}
	*index += 1;

	// feilds
	let mut feilds: Vec<(&str, ASTFormula)> = vec!();
	'feilds: {

		// seperator / close paren
		let end_token = get_next_token(index, contents, "[feild] or ')'")?;
		if *end_token == RawToken::CloseParen {
			*index += 1;
			break 'feilds;
		}

		loop {

			// feild name
			let feild_name_token = get_next_token(index, contents, "[name of feild]")?;
			let RawToken::Name(feild_name) = feild_name_token else {
				return Err(RawCompileError::new_unexpected_token(&contents[*index], "[name of feild]", "while parsing new object feild definition").into());
			};
			*index += 1;
			let feild_is_valid = feilds.iter().all(|feild| feild.0 != *feild_name);

			// colon
			let colon_token = get_next_token(index, contents, "':'")?;
			if *colon_token != RawToken::Colon {
				return Err(RawCompileError::new_unexpected_token(&contents[*index], "':'", "while parsing new object feild definition").into());
			}
			*index += 1;

			// feild value
			let feild_value = parse_formula(index, contents, errors, logger)?;

			// add feild
			if feild_is_valid {
				feilds.push((feild_name, feild_value));
			}

			// skip commas here
			let seperator_token = get_next_token(index, contents, "',' or ')'")?;
			match *seperator_token {
				RawToken::Comma => {
					*index += 1;
					continue;
				}
				RawToken::CloseParen => {
					*index += 1;
					break 'feilds;
				}
				_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "',' or ')'", "while parsing new object feild seperator").into()),
			}

		}
	}

	Ok(ASTFormula::New {
		name: object_name,
		feilds,
	})

}



pub fn parse_formula_item_not<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	
	*index += 1;
	let base = parse_formula_item(index, contents, errors, logger)?;

	Ok(ASTFormula::Not {
		base: box base,
	})
}



pub fn parse_formula_item_parens<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	*index += 1;
	
	let mut items = vec!();
	loop {
		items.push(parse_formula(index, contents, errors, logger)?);
		let next_token = get_next_token(index, contents, "',' or ')'")?;
		match *next_token {
			RawToken::Comma => {
				*index += 1;
				continue;
			}
			RawToken::CloseParen => {
				*index += 1;
				break;
			}
			_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "',' or ')'", "while parsing formula in parentheses").into()),
		}
	}

	Ok(if items.len() == 1 {
		items.pop().unwrap()	
	} else {
		ASTFormula::Tuple(items)
	})
}



pub fn parse_formula_item_formatted_string<'a> (start: &'a str, items: &'a [(Vec<Token>, String)], index: &mut usize, errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	*index += 1;
	let mut output = ASTFormula::String(start);

	for item in items {
		output = ASTFormula::Operation {
			operator: Operator::Concat,
			left: box output,
			right: box parse_formula(&mut 0, &item.0, errors, logger)?,
		};
		output = ASTFormula::Operation {
			operator: Operator::Concat,
			left: box output,
			right: box ASTFormula::String(&item.1),
		};
	}

	Ok(output)
}



pub fn parse_formula_item_function_call<'a> (base: ASTFormula<'a>, has_type_args: bool, index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	*index += 1; // skip open paren / colon

	// type args
	let mut unnamed_type_arg = None;
	let mut named_type_args = vec!();
	if has_type_args {
		(unnamed_type_arg, named_type_args) = parse_type_args(index, contents, errors, logger)?;
		// open paren
		if *get_next_token(index, contents, "'('")? != RawToken::OpenParen {
			return Err(RawCompileError::new_unexpected_token(&contents[*index], "'('", "while parsing generics end").into());
		}
		*index += 1;
	}

	// args
	let mut args: Vec<ASTFormula> = vec!();
	'args: {
		loop {

			// close paren?
			let current_token = get_next_token(index, contents, "[argument] or ')'")?;
			if *current_token == RawToken::CloseParen {
				*index += 1;
				break 'args;
			}

			// arg
			let current_arg = parse_formula(index, contents, errors, logger)?;
			args.push(current_arg);

			// seperator
			let next_token = get_next_token(index, contents, "',' or ')'")?;
			match next_token {
				RawToken::Comma => {
					*index += 1;
					continue;
				}
				RawToken::CloseParen => {
					*index += 1;
					break 'args;
				}
				_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "',' or ')'", "while parsing function argument seperator").into()),
			}

		}
	}

	Ok(ASTFormula::FunctionCall {
		base: box base,
		args,
		type_args: ASTTypeArgs {
			unnamed_arg: unnamed_type_arg,
			named_args: named_type_args,
		},
	})
}



pub fn parse_formula_item_property_query<'a> (base: ASTFormula<'a>, index: &mut usize, contents: &'a [Token], errors: &mut [CompileError], logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {

	*index += 1;
	let property_name_token = get_next_token(index, contents, "[name of property]")?;
	let RawToken::Name(property_name) = property_name_token else {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "[name of property]", "while parsing property query").into());
	};
	*index += 1;

	Ok(ASTFormula::PropertyQuery {
		base: box base,
		key: property_name,
	})
}



pub fn parse_formula_item_index_query<'a> (base: ASTFormula<'a>, index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	
	*index += 1;
	let key = parse_formula(index, contents, errors, logger)?;
	*index += 1;

	Ok(ASTFormula::IndexQuery {
		base: box base,
		key: box key,
	})
}



pub fn parse_formula_item_return_test<'a> (base: ASTFormula<'a>, index: &mut usize, contents: &'a [Token], errors: &mut [CompileError], logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	
	*index += 1;

	Ok(ASTFormula::ReturnTest {
		base: box base,
	})
}



pub fn parse_formula_item_as<'a> (base: ASTFormula<'a>, index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTFormula<'a>, CompileError> {
	
	*index += 1;
	let ast_type = parse_type(index, contents, errors, logger)?;

	Ok(ASTFormula::As {
		base: box base,
		ast_type,
	})
}










pub fn parse_function_definition<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTDefinition<'a>, CompileError> {
	logger.logln(format!("parsing function at {:?}", contents[*index]));
	*index += 1;

	// name
	let mut function_name = match get_next_token(index, contents, "[name of function]")? {
		RawToken::Name(v) => v,
		_ => {
			errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[name of function]", "while parsing function definition").into());
			""
		}
	};
	*index += 1;

	// associated type
	let mut associated_type = None;
	let next_token = get_next_token(index, contents, "'(', '.', or [type data]")?;
	if *next_token == RawToken::Period || *next_token == RawToken::Operator(Operator::LessThan){
		*index -= 1;
		associated_type = Some(parse_type(index, contents, errors, logger)?);
		if *get_next_token(index, contents, "'.'")? != RawToken::Period {
			return Err(RawCompileError::new_unexpected_token(&contents[*index], "'.'", "while parsing function definition after associated type").into());
		}
		*index += 1;
		let function_name_token = get_next_token(index, contents, "[function name]")?;
		function_name = function_name_token.as_name().unwrap_or_else(|| {
			errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[function name]", "while parsing function definition").into());
			""
		});
		*index += 1;
	}

	// args
	let mut args: Vec<ASTFunctionArg> = vec!();
	'args: {

		// open paren
		let args_start_token = get_next_token(index, contents, "'('")?;
		if *args_start_token != RawToken::OpenParen {
			break 'args;
		}
		*index += 1;

		// close paren?
		let current_token = get_next_token(index, contents, "[name of arg] or ')'")?;
		if *current_token == RawToken::CloseParen {
			*index += 1;
			break 'args;
		}

		let mut i = 0;
		'args_loop: loop {

			// arg name & detect 'self'
			let arg_name_token = get_next_token(index, contents, "[name of arg]")?;
			let RawToken::Name(arg_name) = arg_name_token else {
				return Err(RawCompileError::new_unexpected_token(&contents[*index], "[name of arg]", "while parsing function arg definition").into());
			};
			'self_arg: {
				if i == 0 && *arg_name_token == RawToken::Name(String::from("self")) {
					// add self arg
					if associated_type.is_none() {break 'self_arg;}
					let arg_type;
					(arg_type, associated_type) = (associated_type.unwrap(), None);
					args.push(ASTFunctionArg {
						name: "self",
						ast_type: arg_type,
						default: None,
					});
					*index += 1;
					match *get_next_token(index, contents, "',' or ')'")? {
						RawToken::CloseParen => {
							*index += 1;
							break 'args;
						}
						RawToken::Comma => {
							*index += 1;
							i += 1;
							continue 'args_loop;
						}
						_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "',' or ')'", "while parsing function 'self' arg").into()),
					}
				}
			}
			*index += 1;
			let arg_is_valid = args.iter().all(|arg| arg.name != *arg_name);

			// colon
			let colon_token = get_next_token(index, contents, "':'")?;
			if *colon_token != RawToken::Colon {
				return Err(RawCompileError::new_unexpected_token(&contents[*index], "':'", "while parsing function arg definition").into());
			}
			*index += 1;

			// arg type
			let arg_type = parse_type(index, contents, errors, logger)?;

			// default value
			let mut next_token = get_next_token(index, contents, "',', ')', or [default value]")?;
			let default_value = match *next_token {
				RawToken::Int(_) | RawToken::UInt(_) | RawToken::Float(_) | RawToken::Bool(_) | RawToken::String(_) | RawToken::Char(_) => {
					let output = next_token;
					*index += 1;
					next_token = get_next_token(index, contents, "',', ')', or [default value]")?;
					Some(output)
				}
				_ => {None}
			};
			let has_default_value = default_value.is_some();

			// add arg
			if arg_is_valid {
				args.push(ASTFunctionArg {
					name: arg_name,
					ast_type: arg_type,
					default: default_value
				});
			}

			// continuing
			i += 1;
			match *next_token {
				RawToken::Comma => {
					*index += 1;
					continue;
				}
				RawToken::CloseParen => {
					*index += 1;
					break;
				}
				_ => return Err(RawCompileError::new_unexpected_token(
					&contents[*index],
					if has_default_value {"',' or ')'"} else {"',', ')', or [default value]"},
					"while parsing function args seperator"
				).into()),
			}

		}
	}

	// return type
	let mut return_type = ASTType::default();
	let returns_token = get_next_token(index, contents, "'return' or 'end'")?;
	if let RawToken::Name(returns_token_name) = returns_token {
		if *returns_token_name == "returns" {
			*index += 1;
			return_type = parse_type(index, contents, errors, logger)?;
		}
	}

	// statements
    let mut statements = vec!();
    loop {
        let next_token = get_next_token(index, contents, "[statement] or 'end'")?;
        if let RawToken::Name(next_token_name) = next_token {
            if *next_token_name == "end" {
                *index += 1;
                break;
            }
        }
        let new_statement = parse_statement(index, contents, errors, logger)?;
        statements.push(new_statement);
    }

	Ok(ASTDefinition::Function {
		name: function_name,
		associated_type,
		args,
		return_type,
		statements,
	})
}










pub fn parse_object_definition<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTDefinition<'a>, CompileError> {
	logger.logln(format!("parsing function at {:?}", contents[*index]));
	*index += 1;

	// name
	let name = match get_next_token(index, contents, "[name of object]")? {
		RawToken::Name(v) => v,
		_ => {
			errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[name of object]", "while parsing object definition").into());
			""
		}
	};
	*index += 1;

	// open paren
	match *get_next_token(index, contents, "'('")? {
		RawToken::OpenParen => {},
		_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "'('", "while parsing object definition").into()),
	}
	*index += 1;

	let mut feilds = vec!();
	'feilds: loop {

		// close paren?
		if *get_next_token(index, contents, "[name of feild] or ')'")? == RawToken::CloseParen {
			*index += 1;
			break 'feilds;
		}
		
		// feild name
		let feild_name = match get_next_token(index, contents, "[name of feild]")? {
			RawToken::Name(v) => v,
			_ => {
				errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[name of feild]", "while parsing object feild definition").into());
				""
			}
		};
		*index += 1;

		// colon
		match *get_next_token(index, contents, "':'")? {
			RawToken::Colon => {},
			_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "':'", "while parsing object feild definition").into()),
		}
		*index += 1;

		// type
		let ast_type = parse_type(index, contents, errors, logger)?;

		// default value
		let mut next_token = get_next_token(index, contents, "',', '=', or ')'")?;
		let mut default_value = None;
		if *next_token == RawToken::AssignmentOperator(AssignmentOperator::Equals) {
			*index += 1;
			default_value = Some(parse_formula(index, contents, errors, logger)?);
			next_token = get_next_token(index, contents, "',', or ')'")?
		}

		feilds.push(ASTObjectFeild {
			name: feild_name,
			ast_type,
			default_value,
		});

		// seperator / end
		match *next_token {
			RawToken::Comma => {
				*index += 1;
				continue;
			}
			RawToken::CloseParen => {
				*index += 1;
				break 'feilds;
			}
			_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "',', '=', or ')'", "while parsing object feild seperator").into()),
		}

	}

	Ok(ASTDefinition::Object {
		name,
		feilds,
	})
}










pub fn parse_choice_definition<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTDefinition<'a>, CompileError> {
	logger.logln(format!("parsing choice at {:?}", contents[*index]));
	*index += 1;

	// name
	let name = match get_next_token(index, contents, "[name of choice]")? {
		RawToken::Name(v) => v,
		_ => {
			errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[name of choice]", "while parsing choice definition").into());
			""
		}
	};
	*index += 1;

	// open paren
	match *get_next_token(index, contents, "'('")? {
		RawToken::OpenParen => {},
		_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "'('", "while parsing choice definition").into()),
	}
	*index += 1;

	let mut choices = vec!();
	'choices: loop {

		// close paren?
		if *get_next_token(index, contents, "[name of choice] or ')'")? == RawToken::CloseParen {
			*index += 1;
			break 'choices;
		}
		
		// choice name
		let choice_name = match get_next_token(index, contents, "[name of choice]")? {
			RawToken::Name(v) => v,
			_ => {
				errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[name of choice]", "while parsing choice definition").into());
				""
			}
		};
		*index += 1;

		choices.push(choice_name);

		// seperator / end
		match *get_next_token(index, contents, "',' or ')'")? {
			RawToken::Comma => {
				*index += 1;
				continue;
			}
			RawToken::CloseParen => {
				*index += 1;
				break 'choices;
			}
			_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "',', or ')'", "while parsing choice feild seperator").into()),
		}

	}

	Ok(ASTDefinition::Choice {
		name,
		choices,
	})
}










pub fn parse_type_definition<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTDefinition<'a>, CompileError> {
	logger.logln(format!("parsing type at {:?}", contents[*index]));
	*index += 1;

	// name
	let name = match get_next_token(index, contents, "[name of type]")? {
		RawToken::Name(v) => v,
		_ => {
			errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[name of type]", "while parsing type definition").into());
			""
		}
	};
	*index += 1;

	// equal sign
	if *get_next_token(index, contents, "'='")? != RawToken::AssignmentOperator(AssignmentOperator::Equals) {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "'='", "while parsing type definition").into());
	}
	*index += 1;

	// type
	let ast_type = parse_type(index, contents, errors, logger)?;

	Ok(ASTDefinition::Type {
		name,
		ast_type,
	})
}










pub fn parse_const_definition<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTDefinition<'a>, CompileError> {
	logger.logln(format!("parsing const at {:?}", contents[*index]));
	*index += 1;

	// name
	let name = match get_next_token(index, contents, "[name of const]")? {
		RawToken::Name(v) => v,
		_ => {
			errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[name of const]", "while parsing const definition").into());
			""
		}
	};
	*index += 1;

	// equal sign
	if *get_next_token(index, contents, "'='")? != RawToken::AssignmentOperator(AssignmentOperator::Equals) {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "'='", "while parsing const definition").into());
	}
	*index += 1;

	// value
	let value = parse_formula(index, contents, errors, logger)?;

	Ok(ASTDefinition::Const {
		name,
		value,
	})
}










pub fn parse_static_definition<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTDefinition<'a>, CompileError> {
	logger.logln(format!("parsing static at {:?}", contents[*index]));
	*index += 1;

	// name
	let name = match get_next_token(index, contents, "[name of static]")? {
		RawToken::Name(v) => v,
		_ => {
			errors.push(RawCompileError::new_unexpected_token(&contents[*index], "[name of static]", "while parsing static definition").into());
			""
		}
	};
	*index += 1;

	// equal sign
	if *get_next_token(index, contents, "'='")? != RawToken::AssignmentOperator(AssignmentOperator::Equals) {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "'='", "while parsing static definition").into());
	}
	*index += 1;

	// value
	let value = parse_formula(index, contents, errors, logger)?;

	Ok(ASTDefinition::Static {
		name,
		value,
	})
}










pub fn parse_statement<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	logger.logln(format!("parsing statement at {:?}", contents[*index]));

	let first_token = get_next_token(index, contents, "[start of statement] or 'end'")?;
	let RawToken::Name(first_token_name) = first_token else {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "[start of statement] or 'end'", "while parsing next statement").into());
	};

	match &**first_token_name {

		"print" => parse_statement_print(index, contents, errors, logger),
		"throw" => parse_statement_throw(index, contents, errors, logger),
		"crash" => parse_statement_crash(index, contents, errors, logger),
		"assert" => parse_statement_assert(index, contents, errors, logger),
		"todo" => parse_statement_todo(index, contents, errors, logger),

		"var" => parse_statement_var_init(index, contents, errors, logger),

		"if" => parse_statement_if(index, contents, errors, logger),
		"switch" => parse_statement_switch(index, contents, errors, logger),
		"for" => parse_statement_for(index, contents, errors, logger),
		"while" => parse_statement_while(index, contents, errors, logger),
		"loop" => parse_statement_loop(index, contents, errors, logger),
		"break" => {
			*index += 1;
			Ok(ASTStatement::Break)
		}
		"continue" => {
			*index += 1;
			Ok(ASTStatement::Continue)
		}

		"return" => parse_statement_return(index, contents, errors, logger),

		_ => parse_statement_var_assignment_or_function_call(first_token_name, index, contents, errors, logger),
		
	}
}



pub fn parse_statement_print<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	let value = parse_formula(index, contents, errors, logger)?;

	Ok(ASTStatement::Print {value})
}



pub fn parse_statement_throw<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	let value = parse_formula(index, contents, errors, logger)?;

	Ok(ASTStatement::Throw {value})
}



pub fn parse_statement_crash<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	let message = parse_formula(index, contents, errors, logger)?;

	Ok(ASTStatement::Crash {message})
}



pub fn parse_statement_assert<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	let condition = parse_formula(index, contents, errors, logger)?;

	Ok(ASTStatement::Assert {condition})
}



pub fn parse_statement_todo<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	let message = parse_formula(index, contents, errors, logger)?;

	Ok(ASTStatement::Todo {message})
}



pub fn parse_statement_var_init<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	// var names
	let mut var_names = vec!();
	loop {
		let var_name_token = get_next_token(index, contents, "[name of variable]")?;
		let RawToken::Name(var_name) = var_name_token else {
			return Err(RawCompileError::new_unexpected_token(&contents[*index], "[name of variable]", "while parsing 'var' statement's variable names").into());
		};
		var_names.push(var_name.as_str());
		*index += 1;
		let seperator_token = get_next_token(index, contents, "'=' or ','")?;
		match seperator_token {
			RawToken::AssignmentOperator(AssignmentOperator::Equals) => {
				*index += 1;
				break;
			}
			RawToken::Comma => {
				*index += 1;
				continue;
			}
			_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "'=' or ','", "while parsing 'var' statement's variable name seperator").into()),
		}
	}

	// formula
	let formula = parse_formula(index, contents, errors, logger)?;

	Ok(ASTStatement::VarInit {
		var_names,
		value: formula,
	})
}



pub fn parse_statement_if<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	let condition = parse_formula(index, contents, errors, logger)?;

	// then token
	if *get_next_token(index, contents, "'then'")? != RawToken::Name(String::from("then")) {
		return Err(RawCompileError::new_unexpected_token(&contents[*index], "'then'", "while parsing if statement").into());
	}
	*index += 1;

	// true block
	let mut true_block = vec!();
	let has_else = loop {

		// ending token?
		match get_next_token(index, contents, "[statement], 'end', or 'else'")? {
			RawToken::Name(name) if name == "end" => break false,
			RawToken::Name(name) if name == "else" => break true,
			_ => {}
		}

		// statement
		let statement = parse_statement(index, contents, errors, logger)?;
		true_block.push(statement);

	};
	*index += 1;

	// false block
	let mut false_block = vec!();
	if has_else {
		loop {

			// ending token?
			if *get_next_token(index, contents, "[statement] or 'end'")? == RawToken::Name(String::from("end")) {
				break;
			}

			// statement
			let statement = parse_statement(index, contents, errors, logger)?;
			false_block.push(statement);

		}
		*index += 1;
	}

	Ok(ASTStatement::If {
		condition,
		true_block,
		false_block,
	})
}



pub fn parse_statement_switch<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	// switch value
	let switch_value = parse_formula(index, contents, errors, logger)?;

	// cases
	let mut cases = vec!();
	let mut default_case = None;
	'switch: loop {

		// ending token?
		if *get_next_token(index, contents, "[case] or 'end'")? == RawToken::Name(String::from("end")) {
			*index += 1;
			break 'switch;
		}

		// case value
		let case_token_index = *index;
		let case_value = parse_formula(index, contents, errors, logger)?;

		// colon
		if *get_next_token(index, contents, "':'")? != RawToken::Colon {
			return Err(RawCompileError::new_unexpected_token(&contents[*index], "':'", "while parsing switch statement case").into());
		}

		// block
		let mut block = vec!();
		'block: loop {

			// ending token?
			if *get_next_token(index, contents, "[statement] or 'end'")? == RawToken::Name(String::from("end")) {
				*index += 1;
				break 'block;
			}

			// statement
			let statement = parse_statement(index, contents, errors, logger)?;
			block.push(statement);

		}

		if case_value == ASTFormula::Name("default") {
			if default_case.is_some() {
				errors.push(RawCompileError::MultipleDefaultCases {location: contents[case_token_index].clone()}.into());
			}
			default_case = Some(block);
		} else {
			cases.push((case_value, block));
		}

	}

	logger.print_all();
	Ok(ASTStatement::Switch {switch_value, cases, default_case})
}



pub fn parse_statement_for<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	logger.print_all();
	todo!("'for' statement");
}



pub fn parse_statement_while<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	logger.print_all();
	todo!("'while' statement");
}



pub fn parse_statement_loop<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	// false block
	let mut block = vec!();
	loop {

		// ending token?
		if *get_next_token(index, contents, "[statement] or 'end'")? == RawToken::Name(String::from("end")) {
			*index += 1;
			break;
		}

		// statement
		let statement = parse_statement(index, contents, errors, logger)?;
		block.push(statement);

	}

	Ok(ASTStatement::Loop {
		block,
	})
}



pub fn parse_statement_return<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	return Ok(ASTStatement::Return {value: if *get_next_token(index, contents, "[return value] or 'end'")? == RawToken::Name(String::from("end")) {
		*index -= 1;
		None
	} else {
		Some(parse_formula(index, contents, errors, logger)?)
	}})
}



pub fn parse_statement_var_assignment_or_function_call<'a> (first_token: &'a str, index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<ASTStatement<'a>, CompileError> {
	*index += 1;

	let var_queries = parse_var_queries(index, contents, errors, logger)?;

	let next_token = &contents[*index].token;
	Ok(match next_token {

		RawToken::AssignmentOperator(assignment_operator) => {
			*index += 1;
			let value = parse_formula(index, contents, errors, logger)?;
			ASTStatement::VarAssignment {
				start_name: first_token,
				var_queries,
				operator: assignment_operator.clone(),
				value,
			}
		}

		RawToken::OpenParen => {
			*index += 1;
			let mut args = vec!();
			'args: loop {

				// end?
				if *get_next_token(index, contents, "[argument] or ')'")? == RawToken::CloseParen {
					*index += 1;
					break 'args;
				}

				args.push( parse_formula(index, contents, errors, logger)?);

				match *get_next_token(index, contents, "',' or ')'")? {
					RawToken::Comma => {
						*index += 1;
						continue;
					}
					RawToken::CloseParen => {
						*index += 1;
						break 'args;
					}
					_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "',' or ')'", "while parsing function call args seperator").into()),
				}

			}
			println!("end token: {:?}", contents[*index]);
			ASTStatement::FunctionCall {
				start_name: first_token,
				var_queries,
				args,
			}
		}

		_ => unreachable!(),
	})

}



pub fn parse_var_queries<'a> (index: &mut usize, contents: &'a [Token], errors: &mut Vec<CompileError>, logger: &mut Logger) -> Result<Vec<VarQuery<'a>>, CompileError> {
	let mut output = vec!();

	loop {
		match *get_next_token(index, contents, "'=', '(', '.', or '['")? {

			RawToken::AssignmentOperator(_) => break,

			RawToken::OpenParen => break,

			RawToken::Period => {
				*index += 1;
				let RawToken::Name(feild_name) = get_next_token(index, contents, "[name of feild]")? else {
					*index += 1;
					errors.push(RawCompileError::new_invalid_token_type(&contents[*index], "Name").into());
					continue;
				};
				output.push(VarQuery::Feild(feild_name));
				*index += 1;
			}

			RawToken::OpenSquareBracket => {
				*index += 1;
				let key = parse_formula(index, contents, errors, logger)?;
				output.push(VarQuery::Index(key));
			}

			_ => return Err(RawCompileError::new_unexpected_token(&contents[*index], "'=', '(', '.', or '['", "while parsing start of statement").into()),
		}
	}

	Ok(output)
}
