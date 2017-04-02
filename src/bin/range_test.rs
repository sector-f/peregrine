extern crate hyper;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper::header::*;

extern crate hyper_native_tls;
use hyper_native_tls::NativeTlsClient;
use hyper::client::IntoUrl;

extern crate clap;
use clap::{App, Arg};

use std::path::PathBuf;
use std::thread;
use std::fs::File;
use std::io::{Read, Write, copy};

#[allow(dead_code)]
enum Download {
    Single {
        path: PathBuf,
    },
    Fragmented {
        path: PathBuf,
        size: u64,
        parts: u32,
    },
}

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


    let urls = matches.values_of("url").unwrap();
    for url_string in urls {
        let url = url_string.into_url().unwrap();

        let connector = HttpsConnector::new(NativeTlsClient::new().unwrap());
        let client = Client::with_connector(connector);

        let identity = QualityItem::new(Encoding::Identity, Quality(1000));
        let response = client.get(url)
            .header(AcceptEncoding(vec![identity]))
            .send().unwrap();

        // .header(Range::Bytes(vec![ByteRangeSpec::FromTo(0, 128)]))

        let ref headers = response.headers;
        // println!("{}", headers);
        // println!();
        // let content_encoding = headers.get::<ContentEncoding>(); // Must be None or identity

        // See if Accept-Ranges = bytes
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

        println!("Is only identity transfer encoding: {}", is_identity_transfer_encoding);
        println!("Is only identity content encoding: {}", is_identity_content_encoding);
        println!("Accepts byte ranges: {}", accepts_byte_ranges);
        match content_length {
            Some(size) => {
                println!("File size: {} bytes", size);
            },
            None => {
                println!("File size: unknown");
            },
        }

        if is_identity_content_encoding && is_identity_transfer_encoding {
            if accepts_byte_ranges {
                if let Some(size) = content_length {
                    let sections = 4;

                    let size = **size;
                    let partial = size / sections;
                    let mut current_byte: u64 = 0;
                    let mut part_number: u64 = 1;

                    let mut threads = Vec::new();
                    while current_byte + 1 < size {
                        let url = url_string.into_url().unwrap();
                        let next_byte = current_byte + 1;
                        let range =
                            if current_byte == 0 {
                                ByteRangeSpec::FromTo(0, partial)
                            } else if current_byte + partial < size {
                                ByteRangeSpec::FromTo(next_byte, next_byte + partial - 1)
                            } else {
                                ByteRangeSpec::AllFrom(next_byte)
                            };

                        println!("Range: {}", range);

                        let filename = PathBuf::from(format!("testing.part{}", part_number));
                        threads.push(thread::spawn(|| {
                            // To-do: make this work like `curl -O`

                            let connector = HttpsConnector::new(NativeTlsClient::new().unwrap());
                            let client = Client::with_connector(connector);

                            let identity = QualityItem::new(Encoding::Identity, Quality(1000));
                            let mut response = client.get(url)
                                .header(AcceptEncoding(vec![identity]))
                                .header(Range::Bytes(vec![range]))
                                .send().unwrap();

                            // let mut buffer: Vec<u8> = Vec::new();

                            let mut file = File::create(filename).unwrap();
                            let copied_bits = copy(&mut response, &mut file);
                        }));
                        current_byte += partial;
                        part_number += 1;
                    }

                    for thread in threads {
                        thread.join();
                    }
                }
            } else {

            }
         }
    }
}
