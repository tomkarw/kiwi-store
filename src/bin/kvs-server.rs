use clap::{load_yaml, App, ArgMatches};

use kvs::Result;
use std::process;
use std::net::IpAddr;
use std::str::FromStr;

fn main() -> Result<()> {
    let yaml = load_yaml!("kvs-server.yaml");
    let matches = App::from(yaml).get_matches();

    run(&matches)
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    // ip address extraction
    let mut iter = matches.value_of("address").unwrap().splitn(2,":");
    let ip_addr = IpAddr::from_str(iter.next().expect("Bad ip address")).expect("Bad ip address");
    let port = iter.next().expect("Bad ip address");
    println!("{:?}, {}", ip_addr, port);

    // engine extraction
    match matches.value_of("engine").unwrap() {
        "kvs" => println!("kvs"),
        "sled" => println!("sled"),
        _ => {},
    }

    // let mut store = kvs::KvStore::open(".")?;
    Ok(())
}