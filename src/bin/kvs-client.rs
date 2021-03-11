use clap::{load_yaml, App, ArgMatches};

use kvs::Result;
use std::process;
use std::net::{IpAddr, TcpStream};
use std::str::FromStr;
use std::io::{Write, Read};
use log::*;

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

    stream.write("can someone hear me?\n".as_bytes())?;
    stream.flush()?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?; // blocks here, unless server terminate
    info!("{}", buf);
    return  Ok(());

    match matches.subcommand() {
        Some(("set", set_matches)) => {
            let key = set_matches.value_of("key").unwrap().to_owned();
            let value = set_matches.value_of("value").unwrap().to_owned();
            // store.set(key, value)?;
        }
        Some(("get", get_matches)) => {
            let key = get_matches.value_of("key").unwrap().to_owned();
            // let value = store.get(key)?;
            // match value {
            //     Some(value) => println!("{}", value),
            //     None => println!("Key not found"),
            // }
        }
        Some(("rm", remove_matches)) => {
            let key = remove_matches.value_of("key").unwrap().to_owned();
            // if store.remove(key).is_err() {
            //     println!("Key not found");
            //     process::exit(1);
            // }
        }
        _ => {
            println!("No such command");
            process::exit(1);
        },
    }

    Ok(())
}