
use structopt::StructOpt;
use structopt::clap::{App, Shell};

use crate::errors::AppResultU;



#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Shell
    shell: Shell,
    /// Output to
    directory: String,
}


pub fn generate(opt: Opt, mut app: App) -> AppResultU {
    app.gen_completions(env!("CARGO_PKG_NAME"), opt.shell, &opt.directory);
    Ok(())
}
