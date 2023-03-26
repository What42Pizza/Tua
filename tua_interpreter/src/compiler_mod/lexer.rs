use crate::prelude::*;



pub fn lex_tua_file (file: PreprocessedTuaFile, token_combinations: &TokenCombinationNode, path: &Path, logger: &mut Logger) -> (LexedTuaFile, Vec<CompileError>) {
	let errors = vec!();
	let contents = match tokenize_code(&file.contents, token_combinations, 0, false) {
		Ok(v) => v.0,
		Err(error) => return (LexedTuaFile::default(), vec!(error)),
	};
	let contents = contents.into_iter()
		.map(basic_token_data_to_token_data)
		.collect::<Vec<Token>>();
	(LexedTuaFile {contents}, errors)
}



pub fn tokenize_code (contents: &[CharData], token_combinations: &TokenCombinationNode, start: usize, stop_at_curly_backet: bool) -> Result<(Vec<BasicToken>, usize), CompileError> {
	let mut output = vec!();
	let mut index = start;
    let contents_len = contents.len();
	while index < contents_len {
		let current_char = &contents[index];
		if let Some((combined_token, len)) = get_combined_token_at_position(contents, index, token_combinations) {
			output.push(combined_token);
			index += len;
			continue;
		}
		match current_char.char {

            // formatted strings
            '#' if contents[index + 1].char == '"' => {
                let (formatted_string, end_index) = tokenize_formatted_string(contents, index, token_combinations)?;
                output.push(formatted_string);
                index = end_index + 1;
            }

			// strings
			'"' => {
				let quote_end = fns::get_quote_end(contents, index)
					.ok_or_else(|| RawCompileError::NoEndQuote {location: current_char.clone()})?;
				output.push(BasicToken::string_from_chars(&contents[(index+1)..quote_end], current_char.line_num, current_char.char_num));
				index = quote_end + 1;
			}

			// chars
			'\'' => {
				if index + 1 >= contents_len {return Err(RawCompileError::InvalidCharacterDefinition {location: current_char.clone()}.into());}
				let is_escape_code = contents[index + 1].char == '\\';
				let end_char_index = index + if is_escape_code {3} else {2};
				if end_char_index >= contents_len {return Err(RawCompileError::InvalidCharacterDefinition {location: current_char.clone()}.into());}
				output.push(BasicToken::char_from_chars(&contents[(index+1)..end_char_index], current_char.char_num));
				index = end_char_index + 1;
			}

			// names and numbers
			'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
				let word_end = fns::get_word_end(contents, index);
				output.push(BasicToken::name_from_chars(&contents[index..=word_end]));
				index = word_end + 1;
			}

            '}' if stop_at_curly_backet => {
                break;
            }

			// operators & misc
			'(' | ')' | '{' | '}' | '[' | ']' |
			'+' | '-' | '*' | '/' | '^' | '%' |
			'=' | '!' | '>' | '<' |
			'.' | ',' | '?' | ':' | '#' => {
				output.push(BasicToken::special_from_char(current_char));
				index += 1;
			}

			// whitespace
			' ' | '\t' | '\r' | '\n' => {
				index += 1;
			}

			_ => return Err(RawCompileError::InvalidCharacter {location: current_char.clone()}.into()),

		}
	}
	Ok((output, index))
}





fn tokenize_formatted_string (contents: &[CharData], mut index: usize, token_combinations: &TokenCombinationNode) -> Result<(BasicToken, usize), CompileError> {
    let start_token = &contents[index];
    index += 2;
    let mut items = vec!((vec!(), String::new()));

    loop {
        let current_char = contents[index].char;
        match current_char {
            '{' => {
                index += 1;
                let (item, end_index) = tokenize_code(contents, token_combinations, index, true)?;
                items.push((item, String::new()));
                index = end_index + 1;
            }
            '\\' => {
                let next_char = contents[index + 1].char;
                items.last_mut().unwrap().1.push(fns::map_escape_char(next_char));
                index += 2;
            }
            '"' => break,
            _ => {
                items.last_mut().unwrap().1.push(current_char);
                index += 1;
            }
        }
    }

    let start = items.remove(0).1;
    Ok((
        BasicToken {
            token: RawBasicToken::FormattedString {
                start,
                items,
            },
            char_num: start_token.char_num,
            line_num: start_token.line_num,
        },
        index)
    )
}





fn get_combined_token_at_position (contents: &[CharData], starting_index: usize, token_combinations: &TokenCombinationNode) -> Option<(BasicToken, usize)> {
	let mut current_node = token_combinations;
	let mut output = None;
	let starting_token = &contents[starting_index];
	for (i, current_char) in contents.iter().skip(starting_index).enumerate() {
		let current_token_char = current_char.char as usize;
		if current_token_char > 127 {return output;}
		let next_node = &current_node.branches[current_token_char];
		if next_node.is_none() {return output;}
		current_node = next_node.as_ref().unwrap();
		if let Some(final_token) = &current_node.final_token {
			output = Some((BasicToken {
				token: RawBasicToken::Special(final_token.clone()),
				line_num: starting_token.line_num,
				char_num: starting_token.char_num,
			}, i + 1));
		}
	}
	output
}





pub fn basic_token_data_to_token_data (basic_token: BasicToken) -> Token {
    let token = match basic_token.token {
        RawBasicToken::Name            (content) => token_from_name_string(content),
        RawBasicToken::String          (content) => RawToken::String(content),
        RawBasicToken::FormattedString {start, items} => basic_formatted_string_to_formatted_string(start, items),
        RawBasicToken::Char            (content)   => RawToken::Char(content),
        RawBasicToken::Special         (content) => token_from_special_str(&content),
    };
    Token {
        token,
        line_num: basic_token.line_num,
        char_num: basic_token.char_num,
    }
}



pub fn token_from_name_string (input: String) -> RawToken {
    if let Ok(value) = input.parse::<i64>() {
        return RawToken::Int(value);
    }
    if let Ok(value) = input.parse::<u64>() {
        return RawToken::UInt(value);
    }
    if let Ok(value) = input.parse::<f64>() {
        return RawToken::Float(value);
    }
    match &*input {
        "true" => RawToken::Bool(true),
        "false" => RawToken::Bool(false),
        "and" => RawToken::Operator(Operator::And),
        "or" => RawToken::Operator(Operator::Or),
        "xor" => RawToken::Operator(Operator::Xor),
        "not" => RawToken::Operator(Operator::Not),
        "as" => RawToken::Operator(Operator::As),
        _ => RawToken::Name(input),
    }
}



pub fn basic_formatted_string_to_formatted_string (start: String, items: Vec<(Vec<BasicToken>, String)>) -> RawToken {
    let items: Vec<(Vec<Token>, String)> = items.into_iter()
        .map(|(current_arg_content, next_string)| {
            let new_arg_content = current_arg_content.into_iter()
                .map(basic_token_data_to_token_data)
                .collect();
            (new_arg_content, next_string)
        })
        .collect();
    RawToken::FormattedString {start, items}
}



pub fn token_from_special_str (input: &str) -> RawToken {
    if input.len() == 1 {
        let simple_match = match input.chars().next().unwrap() {
            '(' => RawToken::OpenParen,
            ')' => RawToken::CloseParen,
            '[' => RawToken::OpenSquareBracket,
            ']' => RawToken::CloseSquareBracket,
            '}' => RawToken::OpenCurlyBracket,
            '{' => RawToken::CloseCurlyBracket,
            '.' => RawToken::Period,
            ',' => RawToken::Comma,
            '?' => RawToken::QuestionMark,
            ':' => RawToken::Colon,
            '#' => RawToken::Octothorp,
            '+' => RawToken::Operator(Operator::Plus),
            '-' => RawToken::Operator(Operator::Minus),
            '*' => RawToken::Operator(Operator::Times),
            '/' => RawToken::Operator(Operator::Divide),
            '^' => RawToken::Operator(Operator::Power),
            '%' => RawToken::Operator(Operator::Modulo),
            '>' => RawToken::Operator(Operator::GreaterThan),
            '<' => RawToken::Operator(Operator::LessThan),
            '=' => RawToken::AssignmentOperator(AssignmentOperator::Equals),
            _ => RawToken::Char(' '),
        };
        if simple_match != RawToken::Char(' ') {return simple_match;}
    }
    match input {
        ".." => RawToken::Operator(Operator::Concat),
        "==" => RawToken::Operator(Operator::Equal),
        "!=" => RawToken::Operator(Operator::NotEqual),
        ">=" => RawToken::Operator(Operator::GreaterOrEqual),
        "<=" => RawToken::Operator(Operator::LessOrEqual),
        "<<" => RawToken::Operator(Operator::ShiftLeft),
        ">>" => RawToken::Operator(Operator::ShiftRight),
        "+=" => RawToken::AssignmentOperator(AssignmentOperator::Plus),
        "-=" => RawToken::AssignmentOperator(AssignmentOperator::Minus),
        "*=" => RawToken::AssignmentOperator(AssignmentOperator::Times),
        "/=" => RawToken::AssignmentOperator(AssignmentOperator::Divide),
        "%=" => RawToken::AssignmentOperator(AssignmentOperator::Modulo),
        "..=" => RawToken::AssignmentOperator(AssignmentOperator::Concat),
        "<<=" => RawToken::AssignmentOperator(AssignmentOperator::ShiftLeft),
        ">>=" => RawToken::AssignmentOperator(AssignmentOperator::ShiftRight),
        ".=" => RawToken::AssignmentOperator(AssignmentOperator::Call),
        "++" => RawToken::AssignmentOperator(AssignmentOperator::PlusPlus),
        "--" => RawToken::AssignmentOperator(AssignmentOperator::MinusMinus),
        _ => panic!("unknown special token {input}"),
    }
}
