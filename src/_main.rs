use std::error::Error;
use std::fs;
use std::path::PathBuf;

use dirmerge::Entry;
use anyhow::Result;

fn main() -> Result<()> {
    fs::create_dir("test");
    fs::create_dir("test/test");
    fs::create_dir("test/test1");
    fs::File::create("./test/1.txt");
    fs::File::create("./test/test/1.txt");
    fs::File::create("./test/test1/1.txt");
    fs::create_dir("test2");
    fs::File::create("./test2/2.txt");

    dirmerge::Dir{path: PathBuf::from("./test")}.mov(&PathBuf::from("./test2"), false)?;

    Ok(())
}