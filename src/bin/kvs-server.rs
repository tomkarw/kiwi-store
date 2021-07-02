use clap::{load_yaml, App, ArgMatches};

use kvs::{Error, KvStore, KvsEngine, Result, SledKvsEngine};
use log::info;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{env, fs, str};

static DB_PATH: &str = "./database";

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

    run(&matches)
}

fn run(matches: &ArgMatches) -> Result<()> {
    let addr = matches.value_of("address").unwrap();
    let engine = matches.value_of("engine").unwrap();

    info!(
        "kvs-server v{} running at {}",
        env!("CARGO_PKG_VERSION"),
        addr
    );

    if !Path::new(DB_PATH).exists() {
        fs::create_dir(DB_PATH)?
    }

    match engine {
        "kvs" => {
            if Path::new(DB_PATH).join("db").exists() {
                return Err(Error::Other("sled database already exists".to_owned()));
            }
            start_listening(&mut KvStore::open(DB_PATH)?, addr)
        }
        "sled" => {
            if Path::new(DB_PATH).join("kvs.db").exists() {
                return Err(Error::Other("kvs database already exists".to_owned()));
            }
            start_listening(&mut SledKvsEngine::open(DB_PATH)?, addr)
        }
        _ => {
            return Err(Error::Other(
                "unknown engine option, must be one of: kvs, sled".to_owned(),
            ))
        }
    }?;

    Ok(())
}

fn start_listening<E: KvsEngine>(mut store: &mut E, addr: &str) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        handle_connection(&mut store, stream?)?;
    }

    Ok(())
}

fn handle_connection<E: KvsEngine>(store: &mut &mut E, mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    // TODO(clippy): read amount is not handled. Use `Read::read_exact` instead
    stream.read(&mut buffer)?;
    let buffer = str::from_utf8(&buffer).unwrap();

    let mut buffer_iter = buffer.lines();
    let verb = buffer_iter.next().expect("no input");

    let response = if verb == "GET" {
        let key = buffer_iter.next().expect("Key was not provided");
        match store.get(key.to_owned()) {
            Ok(result) => match result {
                Some(value) => format!("OK {}\n", value),
                None => String::from("EMPTY\n"),
            },
            Err(error) => format!("ERROR {}\n", error),
        }
    } else if verb == "SET" {
        let key = buffer_iter.next().expect("Key was not provided");
        let value = buffer_iter.next().expect("Value was not provided");
        match store.set(key.to_owned(), value.to_owned()) {
            Ok(()) => String::from("OK\n"),
            Err(error) => format!("ERROR {}\n", error),
        }
    } else if verb == "RM" {
        let key = buffer_iter.next().expect("Key was not provided");
        match store.remove(key.to_owned()) {
            Ok(()) => String::from("OK\n"),
            Err(error) => format!("ERROR {}\n", error),
        }
    } else {
        String::from("ERROR Unrecognised action verb\n")
    };

    info!("{}", response);
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
