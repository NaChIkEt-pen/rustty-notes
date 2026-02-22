use clap::Parser;

#[derive(PartialEq)]
pub enum Focus {
    Editor,
    Tree,
}

#[derive(Parser)]
pub struct Args {
    pub path: Option<std::path::PathBuf>,
}
