extern crate hyper;
use hyper::Client;
use hyper::net::{HttpsConnector, HttpConnector};
use hyper::header::*;

extern crate hyper_native_tls;
use hyper_native_tls::NativeTlsClient;
use hyper::client::IntoUrl;

extern crate clap;
use clap::{App, Arg};

extern crate threadpool;
use threadpool::ThreadPool;

use std::path::PathBuf;
use std::thread;
use std::fs::File;
use std::io::{Read, Write, copy, stderr};
use std::default::Default;
use std::sync::mpsc;

pub mod dl;
use dl::*;

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

fn get_client() -> Client {
    match NativeTlsClient::new() {
        Ok(tls) => {
            let connector = HttpsConnector::new(tls);
            Client::with_connector(connector)
        },
        Err(e) => {
            let _ = writeln!(stderr(), "Not using TLS due to error: {}", e);
            Client::with_connector(HttpConnector::default())
        },
    }
}

fn get_download(url: hyper::Url, name: Option<PathBuf>, sections: u64) -> Result<Download, ()> {
    let client = get_client();
    let identity = QualityItem::new(Encoding::Identity, Quality(1000));
    let response = client.get(url.clone())
        .header(AcceptEncoding(vec![identity]))
        .send().unwrap();

    let ref headers = response.headers;

    let accepts_byte_ranges = match headers.get::<AcceptRanges>() {
        Some(ranges) => {
            ranges.contains(&RangeUnit::Bytes)
        },
        None => {
            false
        },
    };

    // See if Content-Encoding is (only) identity
    let is_identity_content_encoding = match headers.get::<ContentEncoding>() {
        Some(encodings) => {
            if encodings.len() == 1 {
                encodings[0] == Encoding::Identity
            } else {
                false
            }
        },
        None => {
            true
        },
    };

    let is_identity_transfer_encoding = match headers.get::<TransferEncoding>() {
        Some(encodings) => {
            if encodings.len() == 1 {
                encodings[0] == Encoding::Identity
            } else {
                false
            }
        },
        None => {
            true
        },
    };

    // Get file size in bytes as Option<u64>
    let content_length = headers.get::<ContentLength>();

    if is_identity_content_encoding && is_identity_transfer_encoding {
        if accepts_byte_ranges {
            if let Some(size) = content_length {
                let mut byte_ranges = Vec::new();

                let size = **size;
                let partial = size / sections;
                let mut current_byte: u64 = 0;
                let mut part_number: u64 = 1;

                while current_byte + 1 < size {
                    let next_byte = current_byte + 1;
                    byte_ranges.push(
                        if current_byte == 0 {
                            ByteRangeSpec::FromTo(0, partial)
                        } else if current_byte + partial < size {
                            ByteRangeSpec::FromTo(next_byte, next_byte + partial - 1)
                        } else {
                            ByteRangeSpec::AllFrom(next_byte)
                        }
                    );
                }
                Ok(Download::new(url.clone(), name, Some(byte_ranges)))
            } else { // Server didn't provide content length
                Ok(Download::new(url.clone(), name, None))
            }
        } else { // Server doesn't accept byte ranges
            Ok(Download::new(url.clone(), name, None))
        }
    } else {
        Err(())
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

    let max_connections = 8;
    let sections = 3;
    let threadpool = ThreadPool::new(max_connections);
    let (tx, rx) = mpsc::channel::<()>();

    //////////////////////
    // Response headers //
    //////////////////////
    // Accept-Ranges MUST be bytes
    // Content-Length MUST be present
    // Transfer-Encoding MUST NOT be chunked

    /////////////////////
    // Request headers //
    /////////////////////
    // Range; MUST use bytes
    // Accept-Encoding MUST be identity (for now)

    let mut downloads: Vec<Download> = Vec::new();

    let urls = matches.values_of("url").unwrap();

    // Just do this synchronously for now
    // Switch to futures later
    for url_string in urls {
        let url = url_string.into_url().unwrap();
        let download = get_download(url, None, 3);
        if let Ok(dl) = download {
            downloads.push(dl);
        }
    }

    for dl in downloads {
        match dl {
            Download::Partial(part_dl) => {
                // let name = part_dl.name("index.html");

                // let ranges = part_dl.ranges();
                // for i in 0..part_dl.ranges().len() {
                //     let filename = PathBuf::from(format!("{}.part{}", name.display(), i + 1));
                //     let part_dl = part_dl.clone();
                //     threadpool.execute(move || {
                //         let client = get_client();

                //         let identity = QualityItem::new(Encoding::Identity, Quality(1000));
                //         let mut response = client.get(part_dl.url)
                //             .header(AcceptEncoding(vec![identity]))
                //             .header(Range::Bytes(vec![part_dl.ranges()[i]]))
                //             .send().unwrap();

                //         let mut file = File::create(filename).unwrap();
                //         let copied_bits = copy(&mut response, &mut file);
                //     });
                // }
            },
            Download::Full(full_dl) => {
                threadpool.execute(move || {
                    let filename = full_dl.name("index.html");
                    let client = get_client();

                    let identity = QualityItem::new(Encoding::Identity, Quality(1000));
                    let mut response = client.get(full_dl.url)
                        .header(AcceptEncoding(vec![identity]))
                        .send().unwrap();

                    let mut file = File::create(filename).unwrap();
                    let copied_bits = copy(&mut response, &mut file);
                });
            },
        }
    }
}
