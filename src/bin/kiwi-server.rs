use clap::{arg, Command};
use kiwi_proto::kiwi_service_server::{KiwiService, KiwiServiceServer};
use kiwi_proto::{GetReply, GetRequest, RemoveReply, RemoveRequest, SetReply, SetRequest};
use kiwi_store::Result as KvsResult;
use kiwi_store::{Error, KiwiEngine, KiwiStore, SledStore};
use log::{debug, info};

use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::{env, fs, str};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod kiwi_proto {
    tonic::include_proto!("kiwi_store");
}

static DB_PATH: &str = "./database";

#[derive(Debug, Default)]
pub struct Kvs<E>
where
    E: KiwiEngine,
{
    engine: E,
}

impl<E> Kvs<E>
where
    E: KiwiEngine,
{
    fn new(engine: E) -> Self {
        Kvs { engine }
    }
}

#[tonic::async_trait]
impl<E> KiwiService for Kvs<E>
where
    E: KiwiEngine + std::marker::Sync,
{
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetReply>, Status> {
        debug!("got request: {:?}", &request);

        let reply = match self.engine.get(request.into_inner().key).unwrap() {
            Some(value) => GetReply {
                key_found: true,
                value,
            },
            None => GetReply {
                key_found: false,
                value: String::default(),
            },
        };

        Ok(Response::new(reply))
    }

    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetReply>, Status> {
        debug!("got request: {:?}", &request);

        let SetRequest { key, value } = request.into_inner();
        debug!("{key}, {value}");

        self.engine.set(key, value).unwrap();

        let reply = SetReply {};

        Ok(Response::new(reply))
    }

    async fn remove(
        &self,
        request: Request<RemoveRequest>,
    ) -> Result<Response<RemoveReply>, Status> {
        debug!("got request: {:?}", &request);

        let reply = match self.engine.remove(request.into_inner().key) {
            Ok(()) => RemoveReply { key_found: true },
            Err(_) => RemoveReply { key_found: false },
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> KvsResult<()> {
    // set up logger
    stderrlog::new()
        .module(module_path!())
        .quiet(false)
        .verbosity(5)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    // set up argument parsing
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            arg!(-a --addr <ADDRESS> "IP address, either v4 or v6 in format 'IP:PORT'.")
                .required(false)
                .default_value("127.0.0.1:4000"),
        )
        .arg(
            arg!(-e --engine <ENGINE> "Engine used for backend, either 'kvs' or 'sled'.")
                .required(false)
                .default_value("kvs"),
        )
        .get_matches();

    let addr = matches.value_of("addr").unwrap();
    let engine = matches.value_of("engine").unwrap();
    run(addr, engine).await
}

async fn run(address: &str, engine: &str) -> KvsResult<()> {
    info!(
        "{} v{} running at {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        address
    );

    if !Path::new(DB_PATH).exists() {
        fs::create_dir(DB_PATH)?
    }

    match engine {
        "kvs" => {
            if Path::new(DB_PATH).join("db").exists() {
                return Err(Error::Other("sled database already exists".to_owned()));
            }
            let kvs = Kvs::new(KiwiStore::open(DB_PATH)?);
            Server::builder()
                .add_service(KiwiServiceServer::new(kvs))
                .serve(SocketAddr::from_str(address)?)
                .await?;
            Ok(())
        }
        "sled" => {
            if Path::new(DB_PATH).join("kvs.db").exists() {
                return Err(Error::Other("kvs database already exists".to_owned()));
            }
            let kvs = Kvs::new(SledStore::open(DB_PATH)?);
            Server::builder()
                .add_service(KiwiServiceServer::new(kvs))
                .serve(SocketAddr::from_str(address)?)
                .await?;
            Ok(())
        }
        _ => Err(Error::Other(
            "unknown engine option, must be one of: kvs, sled".to_owned(),
        )),
    }
}
