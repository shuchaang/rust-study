use std::str::FromStr;
use clap::Parser;
use anyhow::{anyhow, Result};
use reqwest::Url;

#[derive(Parser,Debug)]
struct Get {
    #[clap(parse(try_from_str = parse_url))]
    url: String
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?; Ok(s.into())
}

#[derive(Parser,Debug)]
struct  Post {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    #[clap(parse(try_from_str=parse_kv_pair))]
    body: Vec<KvPair>
}

#[derive(Debug)]
struct KvPair {
    k: String,
    v: String
}

impl FromStr for KvPair{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = ||anyhow!(format!("{} is not a valid key value pair", s));
        Ok(Self{
            k:(split.next().ok_or_else(err)?).to_string(),
            v:(split.next().ok_or_else(err)?).to_string()
        })
    }
}
fn parse_kv_pair(s: &str) -> Result<KvPair> { s.parse()}

#[derive(Parser,Debug)]
enum SubCommand {
    Get(Get),
    Post(Post)
}

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Channy")]
struct Opts{
    #[clap(subcommand)]
    subcommand: SubCommand
}


fn main() {
    let opts: Opts = Opts::parse();
    println!("{:?}",opts);
}
