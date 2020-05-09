use clap::{Arg, App, SubCommand};

mod pass;
mod commands;

const DEFAULT_PW_SIZE: usize = 20;

fn main() {

    let matches = App::new("rpass")
        .version("0.1")
        .author("Tibor Schneider <tiborschneider@bluewin.ch>")
        .about("Manage pass without leaking information")
        .subcommand(
            SubCommand::with_name("interactive")
                .about("Copy username or password to clipboard using interactive dmenu")
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Print all entry information")
                .arg(Arg::with_name("path")
                     .short("p")
                     .long("path")
                     .value_name("PATH")
                     .help("path to the key to delete")
                     .takes_value(true))
                .arg(Arg::with_name("uuid")
                     .short("u")
                     .long("uuid")
                     .value_name("UUID")
                     .help("uuid of the key to delete")
                     .takes_value(true)
                     .conflicts_with("path"))
        )
        .subcommand(
            SubCommand::with_name("mv")
                .about("Rename a specific key")
                .arg(Arg::with_name("path")
                     .short("p")
                     .long("path")
                     .value_name("PATH")
                     .help("path to the key")
                     .takes_value(true))
                .arg(Arg::with_name("uuid")
                     .short("u")
                     .long("uuid")
                     .value_name("UUID")
                     .help("uuid of the key")
                     .takes_value(true)
                     .conflicts_with("path"))
                .arg(Arg::with_name("dst")
                     .short("d")
                     .long("dst")
                     .value_name("DESTINATION")
                     .help("Path to move the old key to")
                     .takes_value(true))
        )
        .subcommand(
            SubCommand::with_name("insert")
                .about("Insert a new key")
                .arg(Arg::with_name("path")
                     .short("d")
                     .long("path")
                     .value_name("PATH")
                     .help("path under which to store the new key")
                     .takes_value(true))
                .arg(Arg::with_name("username")
                     .short("u")
                     .long("user")
                     .value_name("USERNAME")
                     .help("username for the key")
                     .takes_value(true))
                .arg(Arg::with_name("password")
                     .short("p")
                     .long("password")
                     .value_name("PASSWORD")
                     .help("password for the key")
                     .takes_value(true))
                .arg(Arg::with_name("generate")
                     .short("g")
                     .long("generate")
                     .help("automatically generate a password with PW_LENGTH characters")
                     .takes_value(false)
                     .conflicts_with("password"))
                .arg(Arg::with_name("url")
                     .long("url")
                     .value_name("URL")
                     .help("url for the key")
                     .takes_value(true))
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Delete an existing key")
                .arg(Arg::with_name("path")
                     .short("d")
                     .long("path")
                     .value_name("PATH")
                     .help("path to the key to delete")
                     .takes_value(true))
                .arg(Arg::with_name("uuid")
                     .short("u")
                     .long("uuid")
                     .value_name("UUID")
                     .help("uuid of the key to delete")
                     .takes_value(true)
                     .conflicts_with("path"))
                .arg(Arg::with_name("force")
                     .short("f")
                     .long("force")
                     .help("skip confirmation")
                     .takes_value(false))
        )
        .subcommand(
            SubCommand::with_name("ls")
                .about("Lists all keys in a tree-like structure")
        )
        .get_matches();

    match matches.subcommand() {
        ("interactive", _)     => commands::interactive(),
        ("get",    Some(args)) => commands::get(args.value_of("path"),
                                                args.value_of("uuid")),
        ("mv",     Some(args)) => commands::mv(args.value_of("path"),

                                               args.value_of("uuid"),
                                               args.value_of("dst")),
        ("insert", Some(args)) => commands::insert(args.value_of("path"),
                                                   args.value_of("username"),
                                                   args.value_of("password"),
                                                   args.value_of("url"),
                                                   match args.is_present("generate") {
                                                       true => Some(DEFAULT_PW_SIZE),
                                                       false => None
                                                   }),
        ("rm", Some(args))     => commands::delete(args.value_of("path"),
                                                   args.value_of("uuid"),
                                                   args.is_present("force")),
        ("ls",   _)            => commands::list(),
        _                      => println!("{}", matches.usage())
    }

}
