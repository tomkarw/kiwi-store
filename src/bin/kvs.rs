use clap::{load_yaml, App, ArgMatches};
use std::error::Error;
use std::process;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    if let Err(e) = run(&matches) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

    eprintln!("unimplemented");
    process::exit(1);
}

pub fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut store = kvs::KvStore::new();

    match matches.subcommand() {
        Some(("set", set_matches)) => {
            let key = set_matches.value_of("key").unwrap();
            let value = set_matches.value_of("value").unwrap();
            store.set(key, value);
        }
        Some(("get", get_matches)) => {
            let key = get_matches.value_of("key").unwrap();
            store.get(key);
        }
        Some(("rm", remove_matches)) => {
            let key = remove_matches.value_of("key").unwrap();
            store.remove(key);
        }
        _ => panic!(),
    }
    Ok(())
}
