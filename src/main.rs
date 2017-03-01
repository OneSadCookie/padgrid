#![feature(conservative_impl_trait)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate futures;
extern crate hyper;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate regex;
extern crate rocket;
extern crate tokio_core;

use futures::{ Future, Stream };
use futures::future;
use rocket::http::ContentType;
use rocket::response::Content;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use regex::Regex;

struct PadGridError;

impl From<io::Error> for PadGridError {

    fn from(_: io::Error) -> PadGridError {
        PadGridError
    }

}

impl From<hyper::Error> for PadGridError {

    fn from(_: hyper::Error) -> PadGridError {
        PadGridError
    }

}

fn cache_path_for_id(id: usize) -> PathBuf {
    format!("cache/{}.png", id).into()
}

fn read_icon_cache(id: usize) -> impl Future<Item=File, Error=PadGridError> {
    future::result(std::fs::File::open(cache_path_for_id(id)).map_err(From::from))
}

fn write_cache_for(id: usize, res: hyper::client::Response) -> impl Future<Item=(), Error=PadGridError> {
    future::result(std::fs::File::create(cache_path_for_id(id))).from_err().and_then(move |cache| {
        res.body().fold(cache, |mut cache, chunk| {
            match cache.write_all(&chunk) {
                Ok(_)    => { Ok(cache) }
                Err(err) => { Err(err) }
            }
        }).from_err()
    }).and_then(|_|{Ok(())})
}

fn download_icon(
    id: usize,
    handle: tokio_core::reactor::Handle
) -> impl Future<Item=File, Error=PadGridError> {
    let url_string = format!("http://www.puzzledragonx.com/en/img/book/{}.png", id);
    let url = hyper::Url::parse(&url_string).unwrap();
    let client = hyper::Client::new(&handle);
    info!(target: "_", "{} not cached; downloading {}", id, url_string);
    client.get(url).from_err().and_then(move |res| {
        if !res.status().is_success() {
            warn!(target: "_", "Received {} downloading {}", res.status(), url_string);
            return Err(PadGridError)
        }
        Ok(res)
    }).and_then(move |res| {
        write_cache_for(id, res)
    }).and_then(move |_| {
        info!(target: "_", "Download succeeded, reopening cache file");
        read_icon_cache(id)
    }).or_else(move |_| {
        warn!(target: "_", "Failed to cache {}", id);
        let _ = std::fs::remove_file(cache_path_for_id(id));
        Err(PadGridError)
    })
}

fn monster_icon_file(
    id: usize,
    handle: tokio_core::reactor::Handle
) -> impl Future<Item=File, Error=PadGridError> {
    read_icon_cache(id).or_else(move |_| {
        download_icon(id, handle)
    })
}

#[get("/monsters/<filename>")]
fn monster(filename: &str) -> Option<Content<File>> {
    lazy_static! {
        static ref FILENAME_RE: Regex = Regex::new(r"^(\d+)(?i:\.png)?$").unwrap();
    }
    let id: usize = match FILENAME_RE.captures(filename) {
        Some(captures) => {
            captures[1].parse().unwrap()
        }
        None => { return None }
    };

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    core.run(monster_icon_file(id, handle)).ok().map(|file| {
        Content(ContentType::PNG, file)
    })
}

#[get("/grid/<description>")]
fn grid(description: &str) -> Option<String> {
    lazy_static! {
        static ref STRIP_EXT_RE: Regex = Regex::new(r"^(.*?)(?i:\.png)?$").unwrap();
    }
    let description = &STRIP_EXT_RE.captures(description).unwrap()[1];
    let monsters = description.split(";").map(|row| {
        row.split(",").map(|s| { s.parse::<usize>() }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let rows = monsters.len();
    let cols = monsters.iter().map(|row| { row.len() }).max().unwrap_or(0);

    Some(format!("{}x{}: {:?}", cols, rows, monsters))
}

fn main() {
    rocket::ignite().mount("/", routes![
        grid,
        monster
    ]).launch();
}
