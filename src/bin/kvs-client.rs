use clap::{load_yaml, App, ArgMatches};

use kvs::Result;
// use log::info;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process;
use std::str;

fn main() -> Result<()> {
    // set up logger
    stderrlog::new()
        .module(module_path!())
        .quiet(false)
        .verbosity(5)
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .unwrap();

    // set up argument parsing
    let yaml = load_yaml!("kvs-client.yaml");
    let matches = App::from(yaml).get_matches();

    run(&matches)
}

pub fn run(matches: &ArgMatches) -> Result<()> {

    let (action, subcommand_matches) = matches.subcommand().unwrap_or_else(|| {
        println!("No such command");
        process::exit(1);
    });

    let address = subcommand_matches.value_of("address").unwrap();
    let mut stream = TcpStream::connect(address)?;

    match action {
        "get" => {
            let key = subcommand_matches.value_of("key").unwrap().to_owned();
            stream.write_all(format!("GET\n{}\n", key).as_bytes())?;
            stream.flush()?;
        }
        "set" => {
            let key = subcommand_matches.value_of("key").unwrap().to_owned();
            let value = subcommand_matches.value_of("value").unwrap().to_owned();
            stream.write_all(format!("SET\n{}\n{}\n", key, value).as_bytes())?;
            stream.flush()?;
        }
        "rm" => {
            let key = subcommand_matches.value_of("key").unwrap().to_owned();
            stream.write_all(format!("RM\n{}\n", key).as_bytes())?;
            stream.flush()?;
        }
        _ => {
            println!("No such command");
            process::exit(1);
        }
    }

    let mut buffer = [0; 1024];
    // TODO(clippy): read amount is not handled. Use `Read::read_exact` instead
    stream.read(&mut buffer)?;
    let buffer = str::from_utf8(&buffer).unwrap();
    println!("{}", buffer);

    Ok(())
}
