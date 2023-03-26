use crate::prelude::*;
use std::time::Instant;





pub fn compile_from_dir<P: Into<PathBuf>> (dir: P, logger: &mut Logger) -> Result<((), Vec<CompileError>), CompileError> {
	let dir = dir.into();
	logger.logln("Starting compilation...");
	let mut errors: Vec<CompileError> = vec!();
	let total_start = Instant::now();

	let token_combinations = TokenCombinationNode::from_strs(&vec!(
		"..",
		"==",
		"!=",
		">=",
		"<=",
		"<<",
		">>",

		"+=",
		"-=",
		"*=",
		"/=",
		"%=",
		"..=",
		"<<=",
		">>=",
		".=",

		"++",
		"--",
	));

	// load files
	let load_files_start = Instant::now();
	logger.logln("");
	logger.logln("Loading files ...");
	let mut init_logger = Logger::new("Loading Tua files");
    let raw_tua_files = init::load_tua_files_from_dir(dir, &mut init_logger)?;
	logger.join(init_logger);
	logger.logln("done");
	let load_files_time = load_files_start.elapsed();

	// preprocess
	let preprocessing_start = Instant::now();
	logger.logln("");
	logger.logln("Preprocessing files ...");
	let mut preprocessed_tua_files = vec!();
	for (contents, path) in raw_tua_files {
		let mut preprocess_logger = Logger::new("Proprocessing Tua file");
		preprocess_logger.logln(format!("Preprocessing file {path:?}..."));
		match init::preprocess_tua_file(contents, &path, &mut preprocess_logger) {
			Ok(next_file) => preprocessed_tua_files.push((next_file, path)),
			Err(error) => errors.push(error),
		}
		preprocess_logger.logln("done");
		logger.join(preprocess_logger);
	}
	logger.logln("done");
	let preprocessing_time = preprocessing_start.elapsed();

	// lex
	let lexing_start = Instant::now();
	logger.logln("");
	logger.logln("Lexing files ...");
	let mut lexed_files = vec!();
		for (contents, path) in preprocessed_tua_files {
		let mut lex_logger = Logger::new("Lexing Tua file");
		lex_logger.logln(format!("Lexing file {path:?}"));
		let (next_file, mut next_errors) = lexer::lex_tua_file(contents, &token_combinations, &path, &mut lex_logger);
		lexed_files.push((next_file, path));
		errors.append(&mut next_errors);
		lex_logger.logln("done");
		logger.join(lex_logger);
	}
	logger.logln("done");
	let lexing_time = lexing_start.elapsed();

	// parse
	let parsing_start = Instant::now();
	logger.logln("");
	logger.logln("Parsing files ...");
	let mut parsed_files = vec!();
	for (contents, path) in lexed_files.iter() {
		let mut parse_logger = Logger::new("Parsing Tua file");
		parse_logger.logln(format!("Parsing file {path:?}"));
		let (next_file, mut next_errors) = parser::parse_tua_file(contents, path, &mut parse_logger);
		parsed_files.push((next_file, path));
		errors.append(&mut next_errors);
		parse_logger.logln("done");
		logger.join(parse_logger);
	}
	logger.logln("done");
	let parsing_time = parsing_start.elapsed();

	logger.logln("");
	logger.logln("Finished compilation (but not really)");
	let total_time = total_start.elapsed();
	// print
	/**/
	logger.logln("\n\n\nOutput:");
	for (file, path) in &parsed_files {
		logger.logln("\nNext file:");
		for item in &file.definitions {
			logger.logln(format!("{item:#?}"));
		}
	}
	logger.logln("\n\n\nErrors:");
	for error in &errors {
		logger.logln(format!("\nNext error:\n{error}"));
	}
	logger.logln("\n\n");
	logger.logln(format!("    loading: {load_files_time:?}"));
	logger.logln(format!("    preprocessing: {preprocessing_time:?}"));
	logger.logln(format!("    lexing: {lexing_time:?}"));
	logger.logln(format!("    parsing: {parsing_time:?}"));
	logger.logln("");
	logger.logln(format!("Total time: {total_time:?}"));
	logger.logln(format!("Total minus loading: --- {:?} ---", total_time - load_files_time));
	logger.logln("");
	/**/

	Ok(((), errors))
}
