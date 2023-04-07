use std::collections::HashMap;
use std::str::FromStr;
use clap::{Parser};
use anyhow::{anyhow, Result};
use colored::*;
use reqwest::{Url, header, Client, Response};
use mime::Mime;
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

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

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body =  HashMap::new();
    for pair in args.body.iter(){
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(&args.url)
        .json(&body)
        .send()
        .await?;
    Ok(print_resp(resp).await?)
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}",resp.version(),resp.status()).blue();
    println!("{}",status);
}

fn print_headers(resp: &Response) {
    for(name,value) in resp.headers(){
        println!("{}: {:?}",name.to_string().green(),value);
    }
    println!();
}
fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => print_syntect(body, "json"),
        Some(v) if v == mime::TEXT_HTML => print_syntect(body, "html"),

        _=> println!("{}", body),
    }
}
async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime  = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn print_syntect(s: &str, ext: &str) {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension(ext).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(s) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
}


#[tokio::main]
async fn main() -> Result<()>{
    let opts: Opts = Opts::parse();
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);
    headers.insert("X-POWERD-BY", "Rust".parse()?);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let result = match opts.subcommand{
        SubCommand::Get(ref args) => get(client,args).await?,
        SubCommand::Post(ref args) => post(client,args).await?,
    };
    Ok(result)
}

