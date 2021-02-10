use clap::{load_yaml, App, ArgMatches};

use kvs::Result;
use std::process;

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    run(&matches)
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    let mut store = kvs::KvStore::open(".")?;

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
        },
    }
    Ok(())
}