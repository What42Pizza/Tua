use crate::prelude::*;

use std::{io, ops::FromResidual, convert::Infallible, backtrace::Backtrace};





//#[derive(Debug)]
pub struct CompileError {
    raw_error: RawCompileError,
    backtrace: Backtrace,
}



impl std::fmt::Debug for CompileError {
    fn fmt (&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(self, fmt)
    }
}



impl Display for CompileError {
    fn fmt (&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {

        // main error
        writeln!(fmt, "Initial error: {:#?}", self.raw_error)?;
        writeln!(fmt, "Error context:")?;
        
        // backtrace
        let frames: Vec<String> = self.backtrace.frames().iter()
            .map(strip_frame_text)
            .collect();
        let (first, last) = fns::find_first_and_last(&frames, |frame| frame.starts_with("\"tua_interpreter::")).unwrap_or_else(|| {
            let _ = write!(fmt, "[no backtrace]");
            (0, 0)
        });
        for frame in frames.iter().take(last + 1).skip(first + 2) {
            let frame = String::from("\"") + &frame[18..frame.len()];
            let mut frame_items: Vec<&str> = frame.split(' ').collect();
            writeln!(fmt, "{} {} {}", frame_items[0], frame_items[3], frame_items[4])?;
        }

        Ok(())
    }
}

fn strip_frame_text (frame: &std::backtrace::BacktraceFrame) -> String {
    let frame = format!("{frame:?}");
    frame[7..(frame.len() - 3)].to_string()
}



impl From<RawCompileError> for CompileError {
    fn from(input: RawCompileError) -> Self {
        Self {
            raw_error: input,
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<IoError> for CompileError {
    fn from (io_error: IoError) -> Self {
        Self {
            raw_error: RawCompileError::Io {
                source: io_error
            },
            backtrace: Backtrace::capture(),
        }
    }
}





//#[derive(Error, Debug)]
#[derive(Debug)]
pub enum RawCompileError {

    InvalidCharacter {
        location: CharData,
    },

    //#[error("No end was foiund for the string starting at line {line_num} char {char_num}")]
    NoEndQuote {
        location: CharData,
    },

    //#[error("No end was found for the block comment starting at line {line_num} char {char_num}")]
    NoBlockCommentEnd {
        location: CharData,
    },

    //#[error("Invalid character definition at line {line_num} char {char_num}")]
    InvalidCharacterDefinition {
        location: CharData,
    },

    //#[error("Invalid token at line {} char {}.\nExpected {expected}, found {:?}", token.line_num, token.char_num, token.token)]
    UnexpectedToken {
        found_token: Token,
        expected: String,
        context: String,
    },

    UnexpectedEndOfFile {
        proceeding_token: Token,
        expected: String,
    },

    BlockNotClosed {
        location: Token,
    },

    InvalidFunctionName {
        location: Token,
    },

    DuplicateFunctionArg {
        location: Token,
    },

    InvalidTypeName {
        location: Token,
    },

    InvalidTokenType {
        found_token: Token,
        expected_type: String,
    },

    MultipleUnnamedTypes {
        location: Token,
    },

    MultipleDefaultCases {
        location: Token,
    },

    UnfinishedFeature {
        details: String,
    },

    Io {
        source: io::Error,
    },

}



impl RawCompileError {

    pub fn new_unexpected_token (starting_token: &Token, expected: &str, context: &str) -> Self {
        Self::UnexpectedToken {
            found_token: starting_token.clone(),
            expected: expected.to_string(),
            context: context.to_string(),
        }
    }

    pub fn new_unexpected_end_of_file (starting_token: &Token, expected: &str) -> Self {
        Self::UnexpectedEndOfFile {
            proceeding_token: starting_token.clone(),
            expected: expected.to_string(),
        }
    }

	pub fn new_invalid_token_type (token: &Token, expected_type: &str) -> Self {
        Self::InvalidTokenType {
            found_token: token.clone(),
            expected_type: expected_type.to_string(),
        }
	}

}



//impl std::error::Error for RawCompileError {}

impl Display for RawCompileError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}



impl From<RawCompileError> for String {
	fn from (tua_compile_error: RawCompileError) -> String {
		format!("TEMPORARY CODE!!! {tua_compile_error:?}")
	}
}
