use clap::{load_yaml, App, ArgMatches};

use kvs::Result;
use std::process;
use std::net::IpAddr;
use std::str::FromStr;

fn main() -> Result<()> {
    let yaml = load_yaml!("kvs-client.yaml");
    let matches = App::from(yaml).get_matches();

    run(&matches)
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    // ip address extraction
    let mut iter = matches.value_of("address").unwrap().splitn(2,":");
    let ip_addr = IpAddr::from_str(iter.next().expect("Bad ip address")).expect("Bad ip address");
    let port = iter.next().expect("Bad ip address");
    println!("{:?}, {}", ip_addr, port);

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