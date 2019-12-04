
use clap::*;



pub fn build() -> App<'static, 'static> {
    app_from_crate!()
        .subcommand(SubCommand::with_name("interactive")
                    .alias("i")
                    .alias("shell")
                    .about("Interactive shell")
                    .arg(Arg::with_name("prompt")
                         .help("Prompt text")
                         .short("p")
                         .long("prompt")
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("build")
                    .alias("b")
                    .about("Build dictionary")
                    .arg(Arg::with_name("format")
                         .help("Dictionary format")
                         .short("f")
                         .long("format")
                         .takes_value(true))
                    .arg(Arg::with_name("dictionary-file")
                         .required(true)
                         .min_values(1)))
        .subcommand(SubCommand::with_name("export")
                    .about("Export")
                    .arg(Arg::with_name("as-text")
                         .help("Extract words from input as text")
                         .short("t")
                         .long("as-text"))
                    .arg(Arg::with_name("format")
                         .help("Export format")
                         .short("f")
                         .long("format")
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("lemmatize")
                    .alias("lem")
                    .alias("lm")
                    .about("Lemmatize")
                    .arg(Arg::with_name("word")
                         .help("Word")
                         .required(true)))
        .subcommand(SubCommand::with_name("lookup")
                    .alias("l")
                    .about("Lookup")
                    .arg(Arg::with_name("no-color")
                        .long("no-color")
                        .help("No color"))
                    .arg(Arg::with_name("n")
                        .short("n")
                        .takes_value(true)
                        .help("Take only n entries"))
                    .arg(Arg::with_name("word")
                         .help("Word")
                         .required(true)))
        .subcommand(SubCommand::with_name("html")
                    .alias("h")
                    .about("Output HTML fragment")
                    .arg(Arg::with_name("word")
                         .help("Word")
                         .required(true)))
        .subcommand(SubCommand::with_name("server")
                    .alias("s")
                    .about("HTTP Server")
                    .arg(Arg::with_name("gui")
                         .help("Show GTK window")
                         .short("g")
                         .long("gui"))
                    .arg(Arg::with_name("curses")
                         .help("Use curses")
                         .short("c")
                         .long("curses"))
                    .arg(Arg::with_name("ignore")
                         .help("Ignore not found")
                         .short("i")
                         .long("ignore"))
                    .arg(Arg::with_name("print")
                         .help("Prints results to stdout")
                         .short("p")
                         .long("print"))
                    .arg(Arg::with_name("plain")
                         .help("Prints results to stdout without colors")
                         .short("P")
                         .long("plain"))
                    .arg(Arg::with_name("kuru")
                         .help("Enable kuru-kuru head")
                         .short("k")
                         .long("kuru"))
                    .arg(Arg::with_name("font-name")
                         .help("Font name for --gui")
                         .short("f")
                         .long("font-name")
                         .default_value("13.0"))
                    .arg(Arg::with_name("font-size")
                         .help("Font size for --gui")
                         .short("s")
                         .long("font-size")
                         .default_value("13.0"))
                    .arg(Arg::with_name("bind-to")
                         .help("host:port to listen")
                         .required(false)))
        .subcommand(SubCommand::with_name("untypo")
                    .alias("u")
                    .about("Untypo")
                    .arg(Arg::with_name("word")
                         .help("Word")
                         .required(true)))
}
