use clap::{load_yaml, App, ArgMatches};
use kiwi_proto::kiwi_store_server::{KiwiStore, KiwiStoreServer};
use kiwi_proto::{GetReply, GetRequest};
use kvs::Result as KvsResult;
use kvs::{Error, KvStore, KvsEngine, NaiveThreadPool, SledKvsEngine, ThreadPool};
use log::{debug, info};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
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
    E: KvsEngine,
{
    engine: E,
}

impl<E> Kvs<E>
    where
    E: KvsEngine,
{
    fn new(engine: E) -> Self {
        Kvs {
           engine
        }
    }
}

#[tonic::async_trait]
impl<E> KiwiStore for Kvs<E>
where
    E: KvsEngine + std::marker::Sync,
{
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetReply>, Status> {
        debug!("got request: {:?}", &request);

        let reply = GetReply {
            // TODO: return option and don't unwrap here
            value: self
                .engine
                .get(request.into_inner().key)
                .unwrap()
                .unwrap()
                .into(),
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
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .unwrap();

    // set up argument parsing
    let yaml = load_yaml!("kvs-server.yaml");
    let matches = App::from(yaml).get_matches();

    run(&matches).await
}

async fn run(matches: &ArgMatches) -> KvsResult<()> {
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
            let kvs = Kvs::new(KvStore::open(DB_PATH)?);
            Server::builder()
                .add_service(KiwiStoreServer::new(kvs))
                .serve(SocketAddr::from_str(addr)?)
                .await?;
            Ok(())
        }
        "sled" => {
            if Path::new(DB_PATH).join("kvs.db").exists() {
                return Err(Error::Other("kvs database already exists".to_owned()));
            }
            let kvs = Kvs::new(SledKvsEngine::open(DB_PATH)?);
            Server::builder()
                .add_service(KiwiStoreServer::new(kvs))
                .serve(SocketAddr::from_str(addr)?)
                .await?;
            Ok(())
        }
        _ => {
            return Err(Error::Other(
                "unknown engine option, must be one of: kvs, sled".to_owned(),
            ))
        }
    }
}
