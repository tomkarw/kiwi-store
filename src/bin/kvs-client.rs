use clap::{load_yaml, App, ArgMatches};

use color_eyre::Result;
use kiwi_proto::kiwi_service_client::KiwiServiceClient;
use kiwi_proto::{GetReply, GetRequest, RemoveRequest, SetRequest};
use std::process;

pub mod kiwi_proto {
    tonic::include_proto!("kiwi_store");
}

#[tokio::main]
async fn main() -> Result<()> {
    // set up logger
    stderrlog::new()
        .module(module_path!())
        .quiet(false)
        .verbosity(5)
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .unwrap();

    color_eyre::install()?;

    // set up argument parsing
    let yaml = load_yaml!("kvs-client.yaml");
    let matches = App::from(yaml).get_matches();

    run(matches).await
}

async fn run(matches: ArgMatches) -> Result<()> {
    let (action, subcommand_matches) = matches.subcommand().unwrap_or_else(|| {
        println!("No such command");
        process::exit(1);
    });

    let address = subcommand_matches.value_of("address").unwrap();
    let address = format!("http://{address}");

    let mut client = KiwiServiceClient::connect(address).await?;

    match action {
        "get" => {
            let key = subcommand_matches.value_of("key").unwrap().to_owned();
            let request = tonic::Request::new(GetRequest { key });
            let response = client.get(request).await.unwrap();
            let GetReply { key_found, value } = response.into_inner();
            if key_found {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        "set" => {
            let key = subcommand_matches.value_of("key").unwrap().to_owned();
            let value = subcommand_matches.value_of("value").unwrap().to_owned();

            let request = tonic::Request::new(SetRequest { key, value });
            let _response = client.set(request).await;
        }
        "rm" => {
            let key = subcommand_matches.value_of("key").unwrap().to_owned();

            let request = tonic::Request::new(RemoveRequest { key });
            let response = client.remove(request).await.unwrap();
            if !response.into_inner().key_found {
                eprintln!("Key not found");
                process::exit(1);
            }
        }
        _ => {
            println!("No such command");
            process::exit(1);
        }
    };

    Ok(())
}
