// Started 11/30/22
// Last updated 03/09/22



// default rust
#![allow(unused)]
#![warn(unused_must_use)]

// nightly features
#![feature(box_syntax)]
#![feature(try_trait_v2)]
#![feature(backtrace_frames)]



mod compiler_mod;
mod data_mod;
mod fns;
mod logger;
mod additions;
mod prelude;

use prelude::*;

use std::env;



fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let mut path = fns::get_program_dir();
    path.push("input");
    let mut logger = Logger::new("Test compiling Tua code");
    let (_, errors) = match compiler::compile_from_dir(path, &mut logger) {
        Ok(v) => v,
        Err(error) => {
            logger.print_all();
            panic!("Fatal error:\n{error}")
        }
    };
    logger.print_all();
}
