use clap::{load_yaml, App, ArgMatches};

use kvs::Result;
use log::info;
use std::io::Write;
use std::net::TcpStream;
use std::process;

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
    // ip address extraction
    let address = matches.value_of("address").unwrap();

    let mut stream = TcpStream::connect(address)?;

    match matches.subcommand() {
        Some(("get", get_matches)) => {
            let key = get_matches.value_of("key").unwrap().to_owned();
            stream.write_all(format!("GET\n{}\n", key).as_bytes())?;
            stream.flush()?;
        }
        Some(("set", set_matches)) => {
            let key = set_matches.value_of("key").unwrap().to_owned();
            let value = set_matches.value_of("value").unwrap().to_owned();
            stream.write_all(format!("SET\n{}\n{}\n", key, value).as_bytes())?;
            stream.flush()?;
        }
        Some(("rm", remove_matches)) => {
            let key = remove_matches.value_of("key").unwrap().to_owned();
            stream.write_all(format!("RM\n{}\n", key).as_bytes())?;
            stream.flush()?;
        }
        _ => {
            println!("No such command");
            process::exit(1);
        }
    }

    // let mut buf = String::new();
    // stream.read_to_string(&mut buf)?;
    // info!("{}", buf);

    Ok(())
}
