use crate::prelude::*;
use std::{rc::Rc, cell::RefCell};










#[derive(Debug)]
pub struct Environment {

    pub int_8s: VarStack<i8>,
    pub int_16s: VarStack<i16>,
    pub int_32s: VarStack<i32>,
    pub int_64s: VarStack<i64>,
    pub uint_8s: VarStack<u8>,
    pub uint_16s: VarStack<u16>,
    pub uint_32s: VarStack<u32>,
    pub uint_64s: VarStack<u64>,
    pub float_32s: VarStack<f32>,
    pub float_64s: VarStack<f64>,
    pub bools: VarStack<bool>,

    pub strings: VarStack<Rc<RefCell<String>>>,
    pub arrays: VarStack<Rc<RefCell<Vec<usize>>>>,
    //pub hashmaps: Values<UnsafeRc<HashMap<usize, usize>>>,

}



#[derive(Debug)]
pub struct VarStack<T> {
    values: Vec<T>,
    stack_starts: Vec<usize>,
}










#[derive(Debug, Clone, Copy)]
pub struct CharData {
    pub char: char,
    pub line_num: usize,
    pub char_num: usize,
}

impl CharData {
    pub const DEFAULT: CharData = CharData {
        char: ' ',
        line_num: 0,
        char_num: 0,
    };
}

impl Default for CharData {
    fn default() -> Self {
        Self {
            char: ' ',
            line_num: 0,
            char_num: 0,
        }
    }
}





#[derive(Debug)]
pub struct RawTuaFile {
    pub contents: Vec<CharData>,
    pub path: Vec<String>,
}





#[derive(Debug, Default)]
pub struct PreprocessedTuaFile {
	pub contents: Vec<CharData>,
}





#[derive(Debug, Default)]
pub struct LexedTuaFile {
    pub contents: Vec<Token>,
}

#[derive(Debug)]
pub struct BasicToken {
    pub token: RawBasicToken,
    pub line_num: usize,
    pub char_num: usize,
}

impl BasicToken {
    pub fn string_from_chars (chars: &[CharData], line_num: usize, char_num: usize) -> Self {
        Self {
            token: RawBasicToken::String(fns::process_raw_string(chars)),
            line_num,
            char_num,
        }
    }
    pub fn char_from_chars (chars: &[CharData], char_num: usize) -> Self {
        Self {
            token: RawBasicToken::Char(fns::process_raw_string(chars).chars().next().unwrap()),
            line_num: chars[0].line_num,
            char_num,
        }
    }
    pub fn name_from_chars (chars: &[CharData]) -> Self {
        Self {
            token: RawBasicToken::Name(fns::process_raw_string(chars)),
            line_num: chars[0].line_num,
            char_num: chars[0].char_num,
        }
    }
    pub fn special_from_char (char_data: &CharData) -> Self {
        Self {
            token: RawBasicToken::Special(char_data.char.to_string()),
            line_num: char_data.line_num,
            char_num: char_data.char_num,
        }
    }
}

#[derive(Debug)]
pub enum RawBasicToken {
    Name (String),
    String (String),
    FormattedString {start: String, items: Vec<(Vec<BasicToken>, String)>},
    Char (char),
    Special (String),
}










#[derive(Debug)]
pub struct TokenCombinationNode {
    pub branches: Box<[Option<TokenCombinationNode>; 128]>,
    pub final_token: Option<String>,
}



impl TokenCombinationNode {

    pub fn from_strs (combined_tokens: &[&str]) -> TokenCombinationNode {
        let mut main_node = TokenCombinationNode::new();
        for current_token in combined_tokens {
            Self::add_combined_token_to_tree (current_token, &mut main_node);
        }
        main_node
    }

    pub fn add_combined_token_to_tree (token: &str, main_node: &mut TokenCombinationNode) {
        let mut current_node = main_node;
        let token_chars: Vec<char> = token.chars().collect();
        for current_char in token_chars.iter() {
            let current_char_int = *current_char as usize;
            if current_char_int > 127 {panic!("Invalid char passed to create_token_combination_tree(): \"{current_char}\" in token \"{token}\".");}
            
            if current_node.branches[current_char_int].is_none() {
                current_node.branches[current_char_int] = Some(TokenCombinationNode::new());
            }
            current_node = current_node.branches[current_char_int].as_mut().unwrap();
            
        }
        current_node.final_token = Some(token.to_string());
    }

    pub fn new() -> Self {
        Self {
            branches: Box::new(array_init::array_init(|_| None)),
            final_token: None,
        }
    }

}










#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token: RawToken,
    pub line_num: usize,
    pub char_num: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawToken {
    Name (String),
    Int (i64),
    UInt (u64),
    Float (f64),
    Bool (bool),
    String (String),
    FormattedString {start: String, items: Vec<(Vec<Token>, String)>},
    Char (char),
    Operator (Operator),
    AssignmentOperator (AssignmentOperator),
    OpenParen,
    CloseParen,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenCurlyBracket,
    CloseCurlyBracket,
    Period,
    Comma,
    QuestionMark,
    Colon,
    Octothorp,
}



#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Times,
    Divide,
    Power,
    Modulo,
    Concat,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    ShiftLeft,
    ShiftRight,
    And,
    Or,
    Xor,
    Not,
    As,
}

impl Operator {
    pub const MAX_EVAL_LEVEL: usize = 7;
    pub fn get_eval_level(&self) -> usize {
        match self {
            Self::Plus           => 3,
            Self::Minus          => 3,
            Self::Times          => 4,
            Self::Divide         => 4,
            Self::Power          => 6,
            Self::Modulo         => 5,
            Self::Concat         => 2,
            Self::Equal          => 1,
            Self::NotEqual       => 1,
            Self::GreaterThan    => 1,
            Self::LessThan       => 1,
            Self::GreaterOrEqual => 1,
            Self::LessOrEqual    => 1,
            Self::ShiftLeft      => 1,
            Self::ShiftRight     => 1,
            Self::And            => 0,
            Self::Or             => 0,
            Self::Xor            => 0,
            Self::Not            => 0,
            Self::As             => 7,
        }
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentOperator {
    Equals,
    Plus,
    Minus,
    Times,
    Divide,
    Modulo,
    Concat,
    ShiftLeft,
    ShiftRight,
    Call,
    PlusPlus,
    MinusMinus,
}





impl RawToken {
    pub fn as_name (&self) -> Option<&str> {
        match self {
            Self::Name(name) => Some(name),
            _ => None,
        }
    }
}










#[derive(Debug, Default)]
pub struct ParsedTuaFile<'a> {
    pub definitions: Vec<ASTDefinition<'a>>,
}

#[derive(Debug)]
pub enum ASTDefinition<'a> {
    Function {
        name: &'a str,
        associated_type: Option<ASTType<'a>>,
        args: Vec<ASTFunctionArg<'a>>,
        return_type: ASTType<'a>,
        statements: ASTBlock<'a>,
    },
    Object {
        name: &'a str,
        feilds: Vec<ASTObjectFeild<'a>>
    },
    Choice {
        name: &'a str,
        choices: Vec<&'a str>
    },
    Type {
        name: &'a str,
        ast_type: ASTType<'a>,
    },
    Const {
        name: &'a str,
        value: ASTFormula<'a>,
    },
    Static {
        name: &'a str,
        value: ASTFormula<'a>,
    },
}

#[derive(Debug)]
pub struct ASTFunctionArg<'a> {
    pub name: &'a str,
    pub ast_type: ASTType<'a>,
    pub default: Option<&'a RawToken>,
}

#[derive(Debug)]
pub struct ASTObjectFeild<'a> {
    pub name: &'a str,
    pub ast_type: ASTType<'a>,
    pub default_value: Option<ASTFormula<'a>>,
}



pub type ASTBlock<'a> = Vec<ASTStatement<'a>>;

#[derive(Debug)]
pub enum ASTStatement<'a> {

    Print {value: ASTFormula<'a>},
    Throw {value: ASTFormula<'a>},
    Crash {message: ASTFormula<'a>},
    Assert {condition: ASTFormula<'a>},
    Todo {message: ASTFormula<'a>},

    VarInit {var_names: Vec<&'a str>, value: ASTFormula<'a>},
    VarAssignment {start_name: &'a str, var_queries: Vec<VarQuery<'a>>, operator: AssignmentOperator, value: ASTFormula<'a>},

    If {condition: ASTFormula<'a>, true_block: ASTBlock<'a>, false_block: ASTBlock<'a>},
    Switch {switch_value: ASTFormula<'a>, cases: Vec<(ASTFormula<'a>, ASTBlock<'a>)>, default_case: Option<ASTBlock<'a>>},
    For {var_names: Vec<&'a str>, iter: ASTFormula<'a>, block: ASTBlock<'a>},
    While {condition: ASTFormula<'a>, block: ASTBlock<'a>},
    Loop {block: ASTBlock<'a>},
    Break,
    Continue,

    FunctionCall {start_name: &'a str, var_queries: Vec<VarQuery<'a>>, args: Vec<ASTFormula<'a>>},
    Return {value: Option<ASTFormula<'a>>},

}

#[derive(Debug)]
pub enum VarQuery<'a> {
    Feild (&'a str),
    Index (ASTFormula<'a>),
}

#[derive(Debug, PartialEq)]
pub enum ASTFormula<'a> {

    Name (&'a str),

    Int (i64),
    UInt (u64),
    Float (f64),
    Bool (bool),
    String (&'a str),
    Char (char),

    Tuple (Vec<ASTFormula<'a>>),

    Operation {operator: Operator, left: Box<ASTFormula<'a>>, right: Box<ASTFormula<'a>>},
    New {name: &'a str, feilds: Vec<(&'a str, ASTFormula<'a>)>},
    Not {base: Box<ASTFormula<'a>>},
    As {base: Box<ASTFormula<'a>>, ast_type: ASTType<'a>},
    IndexQuery {base: Box<ASTFormula<'a>>, key: Box<ASTFormula<'a>>},
    PropertyQuery {base: Box<ASTFormula<'a>>, key: &'a str},
    ReturnTest {base: Box<ASTFormula<'a>>},
    FunctionCall {base: Box<ASTFormula<'a>>, args: Vec<ASTFormula<'a>>, type_args: ASTTypeArgs<'a>},

}

#[derive(Debug, PartialEq)]
pub struct ASTTypeArgs<'a> {
    pub unnamed_arg: Option<ASTType<'a>>,
    pub named_args: Vec<(&'a str, ASTType<'a>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASTType<'a> {
    pub name: &'a str,
    pub unnamed_type_arg: Option<Box<ASTType<'a>>>,
    pub named_type_args: Vec<(&'a str, ASTType<'a>)>,
}

impl<'a> Default for ASTType<'a> {
    fn default() -> Self {
        Self {
            name: "none",
            unnamed_type_arg: None,
            named_type_args: vec!(),
        }
    }
}



impl<'a> ASTStatement<'a> {
    pub fn default() -> Self {
        Self::Print{value: ASTFormula::Int(0)}
    }
}
