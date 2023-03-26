use std::ffi::OsStr;

use crate::prelude::*;





pub fn load_tua_files_from_dir (dir: PathBuf, logger: &mut Logger) -> Result<Vec<(String, PathBuf)>, CompileError> {
	let mut all_file_paths = fns::get_all_files_in_dir(dir)
		.map_err(|e: IoError| -> CompileError {e.into()})?;
	let mut output = vec!();
	while !all_file_paths.is_empty() {
		let file_path = all_file_paths.pop().unwrap();
		logger.logln(format!("Loading file {file_path:?}..."));
		let file_extension = file_path.extension().map(OsStr::to_str);
		if file_extension != Some(Some("tua")) {println!("{:?}", file_path); continue;}
		let file_contents = fs::read_to_string(&file_path).map_err(|e: IoError| -> CompileError {e.into()})?;
		logger.logln("done");
		output.push((file_contents, file_path));
	}
	Ok(output)
}





pub fn preprocess_tua_file (raw_tua_file: String, path: &Path, logger: &mut Logger) -> Result<PreprocessedTuaFile, CompileError> {
	let raw_tua_file = seperate_file_chars(raw_tua_file)?;
	let contents = match remove_comments(raw_tua_file.contents) {
		Ok(contents) => contents,
		Err(error) => return Ok(PreprocessedTuaFile::default()),
	};
	Ok(PreprocessedTuaFile {contents})
}



pub fn seperate_file_chars (raw_contents: String) -> Result<RawTuaFile, CompileError> {
	let mut contents = vec!();
	let mut line_num = 0;
	let mut char_num = 0;
	for char in raw_contents.chars() {
		match char {
			'\n' => {
				contents.push(CharData {char, line_num, char_num});
				line_num += 1;
				char_num = 0;
			}
			'\r' => {}
			_ => {
				contents.push(CharData {char, line_num, char_num});
				char_num += 1;
			}
		}
	}
	Ok(RawTuaFile {
		contents,
		path: vec!(),
	})
}





fn remove_comments (file_chars: Vec<CharData>) -> Result<Vec<CharData>, CompileError> {
	let mut output = vec!();
	let mut file_chars_iter = file_chars.into_iter().enumerate();
	let mut next_char_to_use = None;
	loop {
		let (i, mut current_char) = next_char_to_use.unwrap_or_else(|| {
			let current_char = file_chars_iter.next();
			current_char.unwrap_or((usize::MAX, CharData::DEFAULT))
		});
		next_char_to_use = None;
		if i == usize::MAX {break;}
		match current_char.char {

			// skip (and append) strings
			'"' => {
				let mut prev_char = CharData::default();
				loop {
					output.push(current_char);
					(_, current_char) = file_chars_iter.next().ok_or_else(|| RawCompileError::NoEndQuote {location: current_char.clone()})?;
					if current_char.char == '"' && prev_char.char != '\\' {
						output.push(current_char);
						break;
					}
					prev_char = current_char;
				}
			}

			// skip comments
			'/' => {
				let next_char = file_chars_iter.next();
				if next_char.is_none() {
					output.push(current_char);
					return Ok(output);
				}
				let (i, mut next_char) = next_char.unwrap();
				match next_char.char {
					'/' => {
						'line_comment: loop {
							if let Some((_, current_char)) = file_chars_iter.next() {
								if current_char.char == '\n' {break 'line_comment;}
							} else {
								return Ok(output);
							}
						}
						next_char.char = '\n';
						output.push(next_char);
					},
					'*' => {
						let mut prev_char = '/';
						'block_comment: loop {
							if let Some((_, current_char)) = file_chars_iter.next() {
								if current_char.char == '/' && prev_char == '*' {break 'block_comment;}
								prev_char = current_char.char;
							} else {
								return Err(RawCompileError::NoBlockCommentEnd {location: current_char.clone()}.into());
							}
						}
						next_char.char = '\n';
						output.push(next_char);
					},
					_ => {
						output.push(current_char);
						next_char_to_use = Some((i + 1, next_char));
						continue;
					},
				}
			}

			_ => {
				output.push(current_char);
			}
			
		}
	}
	Ok(output)
}
