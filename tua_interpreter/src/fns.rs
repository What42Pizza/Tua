use crate::prelude::*;



pub fn find_first_and_last<T> (array: &[T], test: impl Fn(&T) -> bool) -> Option<(usize, usize)> {
    let mut lower = None;
    for (i, item) in array.iter().enumerate() {
        if test(item) {
            lower = Some(i);
            break;
        }
    }
    let Some(lower) = lower else {
        return None;
    };
    let mut higher = 0;
    for (i, item) in array.iter().rev().enumerate() {
        if test(item) {
            higher = (array.len() - i) - 1;
            break;
        }
    }
    Some((lower, higher))
}



pub fn extract_errors<T> (input: Vec<(T, Vec<CompileError>)>, errors: &mut Vec<CompileError>) -> Vec<T> {
    let mut results = vec!();
    for (item, mut item_errors) in input.into_iter() {
        results.push(item);
        errors.append(&mut item_errors);
    }
    results
}



pub fn extract_single_errors<T> (input: Vec<Result<T, CompileError>>, errors: &mut Vec<CompileError>) -> Vec<T> {
    let mut results = vec!();
    for item in input.into_iter() {
        match item {
            Ok(item) => results.push(item),
            Err(error) => errors.push(error),
        }
    }
    results
}



pub fn get_quote_end (contents: &[CharData], quote_start: usize) -> Option<usize> {
	for (i, current_char) in contents.iter().enumerate().skip(quote_start + 1) {
		if current_char.char == '"' {return Some(i);}
	}
	None
}



pub fn get_word_end (contents: &[CharData], word_start: usize) -> usize {
	for (i, current_char) in contents.iter().enumerate() {
		if i <= word_start {continue;}
		match current_char.char {
			'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => continue,
			_ => return i - 1,
		}
	}
	contents.len() - 1
}



pub fn process_raw_string (string: &[CharData]) -> String {
    let mut output = String::from("");
    let mut is_escape_char = false;
    for char_data in string {
        let char = char_data.char;
        if char == '\\' {is_escape_char = true; continue;}
        output.push(if is_escape_char {
            map_escape_char(char)
        } else {
            char
        });
        is_escape_char = false;
    }
    output
}



pub fn map_escape_char (char: char) -> char {
    match char {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '0' => '\0',
        //'\'' => '\'', // both are covered by the _ case
        //'"' => '"',
        _ => char,
    }
}





pub fn get_program_dir() -> PathBuf {
    let mut path = std::env::current_exe()
        .expect("Could not retrieve the path of the current program.");
    path.pop();
    path
}



pub fn get_all_files_in_dir<P: Into<PathBuf>> (path: P) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut folders_to_check = vec!(path.into());
    let mut output = vec!();
    while !folders_to_check.is_empty() {
        let current_folder = folders_to_check.pop().unwrap();
        for current_item in fs::read_dir(current_folder)? {
            let current_item = current_item?.path();
            if current_item.is_dir() {
                folders_to_check.push(current_item);
            } else {
                output.push(current_item);
            }
        }
    }
    Ok(output)    
}



pub fn some_if<T> (condition: bool, some_fn: impl FnOnce() -> T) -> Option<T> {
    if condition {
        Some(some_fn())
    } else {
        None
    }
}
