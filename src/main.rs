use clap::Parser;

use std::fs::{self, DirEntry, Metadata};
use std::os::unix::fs::{DirEntryExt, MetadataExt};
use std::path::Path;

use crate::parser::parse_predicate;
use crate::tokenizer::{Tokens, contains_valid_closing_quote, tokenize};

mod filter;
mod operators;
mod tokenizer;
mod parser;
mod constraints;

#[derive(Parser)]
struct Args {
    #[arg(default_value_t = String::from("."))]
    /// list content of directory specified by path
    path: String,

    #[arg(short, long)]
    /// print all entries of from path
    all: bool, 

    #[arg(short = 'A', long = "almost-all")]
    /// print allmost all entries excluding '.' and '..'
    allmost_all: bool,

    #[arg(short = 'f', long = "filter")]
    /// applies a predicate to each entry. Entries for wich the predocate yields true a included in the output
    /// predicate is expected to be in normal polish notation and every token should be seperated by a single white space
    /// Following operands are accepted:
    /// !: not
    /// ||:  or 
    /// &&: and
    /// ^: xor
    /// =>: conditional
    /// =: biconditional
    filter_npn: Option<String>
}



fn main() {
    let args = Args::parse();
    let raw_predicate = "& & & perm:a:r,u:wx,o:x ! type:d name:test\\. size:=0";

    let tokens =tokenize(raw_predicate).unwrap();

    match parse_predicate(tokens) {
        Ok(p) => {
        if let Ok(entries) = fs::read_dir(args.path) {
            entries
            .filter_map(|entry| entry.ok()) // remove errors from the iterator // remove entries based on the filter criteria
            .filter(|entry| p.eval(entry))
            .for_each(|entry| println!("{} size: {}", entry.file_name().to_string_lossy(), entry.metadata().unwrap().len())); // print entries
        }},
        Err(e) => eprintln!("{e}")
    }
}
