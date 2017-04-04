extern crate hyper;
use hyper::Url;
use hyper::header::ByteRangeSpec;

// use std::path:: PathBuf;
use std::path::{Path, PathBuf};

pub enum Download {
    Partial(PartDl),
    Full(FullDl),
}

impl Download {
    pub fn new(url: hyper::Url, name: Option<PathBuf>, ranges: Option<Vec<ByteRangeSpec>>) -> Self {
        match ranges {
            Some(ranges) => {
                Download::Partial(PartDl::new(url, name, ranges))
            },
            None => {
                Download::Full(FullDl::new(url, name))
            }
        }
    }

    pub fn num_parts(&self) -> usize {
        match self {
            &Download::Partial(ref part_dl) => {
                part_dl.ranges.len()
            },
            &Download::Full(_) => {
                1
            }
        }
    }
}

#[derive(Clone)]
pub struct PartDl {
    pub url: hyper::Url,
    name: Option<PathBuf>,
    ranges: Vec<ByteRangeSpec>,
}

pub struct FullDl {
    pub url: hyper::Url,
    name: Option<PathBuf>,
}

impl PartDl {
    fn new(url: hyper::Url, name: Option<PathBuf>, ranges: Vec<ByteRangeSpec>) -> Self {
        PartDl {
            url: url,
            name: name,
            ranges: ranges,
        }
    }

    pub fn url(&self) -> hyper::Url {
        self.url.clone()
    }

    pub fn ranges(&self) -> Vec<ByteRangeSpec> {
        self.ranges.clone()
    }

    pub fn name(&self, default_index: &str) -> PathBuf {
        match self.name {
            Some(ref name) => {
                PathBuf::from(name)
            },
            None => {
                match self.url.path_segments() {
                    Some(ref segments) => {
                        let segments = segments.clone().collect::<Vec<_>>();
                        let mut last_non_null = default_index;
                        for segment in segments.into_iter().rev() {
                            if segment != "" {
                                last_non_null = segment;
                                break;
                            }
                        }
                        PathBuf::from(last_non_null)
                    },
                    None => {
                        PathBuf::from(default_index)
                    },
                }
            },
        }
    }
}

impl FullDl {
    fn new(url: hyper::Url, name: Option<PathBuf>) -> Self {
        FullDl {
            url: url,
            name: name,
        }
    }

    pub fn name(&self, default_index: &str) -> PathBuf {
        match self.name {
            Some(ref name) => {
                PathBuf::from(name)
            },
            None => {
                match self.url.path_segments() {
                    Some(ref segments) => {
                        let segments = segments.clone().collect::<Vec<_>>();
                        let mut last_non_null = default_index;
                        for segment in segments.into_iter().rev() {
                            if segment != "" {
                                last_non_null = segment;
                                break;
                            }
                        }
                        PathBuf::from(last_non_null)
                    },
                    None => {
                        PathBuf::from(default_index)
                    },
                }
            },
        }
    }
}
