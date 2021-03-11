use clap::{load_yaml, App, ArgMatches};

use kvs::{Result, KvStore};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::str::FromStr;
use log::*;
use std::io::{Read, Write};

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
    let yaml = load_yaml!("kvs-server.yaml");
    let matches = App::from(yaml).get_matches();

    // info!("{}", App::render_version());

    run(&matches)
}

fn run(matches: &ArgMatches) -> Result<()> {
    let addr = matches.value_of("address").unwrap();
    let engine = matches.value_of("engine").unwrap();

    info!("{}, {}", addr, engine);

    match engine {
        "kvs" => (),
        "sled" => (),
        _ => {},
    }

    let mut store = kvs::KvStore::open(".")?;

    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        handle_connection(stream?)?;
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?; // doesn't work with read_to_string

    info!("{:?}", buffer);

    let response = "processed successfully\n";

    stream.write(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
