use clap::{arg, ArgMatches, Command};

use kiwi_store::{KiwiEngine, KiwiStore, Result};
use std::process;

fn main() -> Result<()> {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("set")
                .about("Set value for key.")
                .arg(arg!(<KEY>))
                .arg(arg!(<VALUE>))
                .arg(arg!(-a --addr <ADDRESS> "IP address either v4 or v6 in format 'IP:PORT'")),
        )
        .subcommand(
            Command::new("get")
                .about("Get value for key.")
                .arg(arg!(<KEY>))
                .arg(arg!(-a --addr <ADDRESS> "IP address either v4 or v6 in format 'IP:PORT'")),
        )
        .subcommand(
            Command::new("rm")
                .about("Remove key and value.")
                .arg(arg!(<KEY>))
                .arg(arg!(-a --addr <ADDRESS> "IP address either v4 or v6 in format 'IP:PORT'")),
        )
        .get_matches();

    run(&matches)
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    let store = KiwiStore::open(".")?;

    match matches.subcommand() {
        Some(("set", set_matches)) => {
            let key = set_matches.value_of("key").unwrap().to_owned();
            let value = set_matches.value_of("value").unwrap().to_owned();
            store.set(key, value)?;
        }
        Some(("get", get_matches)) => {
            let key = get_matches.value_of("key").unwrap().to_owned();
            let value = store.get(key)?;
            match value {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        }
        Some(("rm", remove_matches)) => {
            let key = remove_matches.value_of("key").unwrap().to_owned();
            if store.remove(key).is_err() {
                println!("Key not found");
                process::exit(1);
            }
        }
        _ => {
            println!("No such command");
            process::exit(1);
        }
    }
    Ok(())
}
