#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate futures;
extern crate hyper;
#[macro_use] extern crate log;
extern crate rocket;
extern crate tokio_core;

use futures::{ Future, Stream };
use hyper::Client;
use rocket::http::ContentType;
use rocket::response::Content;
use std::fs::File;
use std::io::Write;

#[get("/monsters/<id>")]
fn monster(id: usize) -> Option<Content<File>> {
    let path = format!("cache/{}.png", id);
    let result = std::fs::File::open(&path);
    match result {
        Ok(file) => {
            info!(target: "_", "cached in {}", path);
            return Some(Content(ContentType::PNG, file));
        }
        _        => {
            info!(target: "_", "downloading to {}", path);
        }
    }

    let result = std::fs::File::create(&path);
    let mut cache = match result {
        Ok(file) => { file }
        _        => {
            info!(target: "_", "unable to create {}", path);
            return None
        }
    };

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let client = Client::new(&handle);

    let url_string = format!("http://www.puzzledragonx.com/en/img/book/{}.png", id);
    let url = hyper::Url::parse(&url_string).unwrap();
    let work = client.get(url).and_then(|res| {
        if !res.status().is_success() {
            warn!(target: "_", "http status {} for {}", res.status(), url_string);
            return Err(hyper::Error::Status)
        }
        Ok(res)
    }).and_then(|res| {
        info!(target: "_", "connected; writing to {}", path);
        res.body().for_each(|chunk| {
            cache.write_all(&chunk).map_err(From::from)
        })
    }).and_then(|_| {
        info!(target: "_", "reopening {} to read", path);
        std::fs::File::open(&path).map_err(From::from)
    }).or_else(|err| {
        info!(target: "_", "error writing cache to {}", path);
        let _ = std::fs::remove_file(&path);
        Err(err)
    });

    match core.run(work) {
        Ok(file) => { Some(Content(ContentType::PNG, file)) }
        _        => { None }
    }
}

fn main() {
    rocket::ignite().mount("/", routes![monster]).launch();
}
