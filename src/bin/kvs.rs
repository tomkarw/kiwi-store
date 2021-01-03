use clap::{load_yaml, App, ArgMatches};

use kvs::Result;

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    run(&matches)
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    let mut store = kvs::KvStore::open("kvs.db")?;

    match matches.subcommand() {
        Some(("set", set_matches)) => {
            let key = set_matches.value_of("key").unwrap().to_owned();
            let value = set_matches.value_of("value").unwrap().to_owned();
            store.set(key, value)?;
        }
        Some(("get", get_matches)) => {
            let key = get_matches.value_of("key").unwrap().to_owned();
            store.get(key)?;
        }
        Some(("rm", remove_matches)) => {
            let key = remove_matches.value_of("key").unwrap().to_owned();
            store.remove(key)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}