extern crate hyper;
use hyper::Client;
use hyper::net::HttpsConnector;

extern crate hyper_native_tls;
use hyper_native_tls::NativeTlsClient;
use hyper::client::IntoUrl;

extern crate clap;
use clap::{App, Arg};

fn is_url(val: String) -> Result<(), String> {
    match val.into_url() {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            return Err(format!("{}", e))
        },
    }
}

fn main() {
    let matches = App::new("get_header")
        .arg(Arg::with_name("url")
             .value_name("URL")
             .index(1)
             .required(true)
             .validator(is_url)
             .multiple(true))
        .get_matches();

    for url in matches.values_of("url").unwrap() {
        println!("{}", url);
        let client = Client::with_connector(HttpsConnector::new(NativeTlsClient::new().unwrap()));
        let response = client.get(url).send().unwrap();

        println!("{}", response.status);
        for header in response.headers.iter() {
            print!("{}", header);
        }
        println!();
    }
}
