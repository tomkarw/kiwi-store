use clap::{load_yaml, App, ArgMatches};

use kvs::{KvStore, Result};
use log::info;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

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
        _ => {}
    }

    let mut store = KvStore::open(".")?;

    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        handle_connection(&mut store, stream?)?;
    }

    Ok(())
}

fn handle_connection(store: &mut KvStore, mut stream: TcpStream) -> Result<()> {
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer)?;

    let mut buffer_iter = buffer.lines();
    let verb = buffer_iter.next().expect("no input");
    let response;

    if verb == "GET" {
        let key = buffer_iter.next().expect("Key was not provided");
        match store.get(key.to_string())? {
            Some(value) => response = format!("OK {}\n", value),
            None => response = String::from("EMPTY\n"),
        }
    } else if verb == "SET" {
        let key = buffer_iter.next().expect("Key was not provided");
        let value = buffer_iter.next().expect("Value was not provided");
        match store.set(key.to_string(), value.to_string()) {
            Ok(()) => response = String::from("OK\n"),
            Err(error) => response = format!("ERROR {}\n", error),
        }
    } else if verb == "RM" {
        let key = buffer_iter.next().expect("Key was not provided");
        match store.remove(key.to_string()) {
            Ok(()) => response = String::from("OK\n"),
            Err(error) => response = format!("ERROR {}\n", error),
        }
    } else {
        response = String::from("ERROR Unrecognised action verb\n")
    }

    info!("{}", response);
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
