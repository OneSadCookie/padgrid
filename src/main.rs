#![feature(conservative_impl_trait)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate futures;
extern crate hyper;
extern crate image;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate regex;
extern crate rocket;
extern crate tokio_core;

use futures::{ Future, Stream };
use futures::future;
use image::{ DynamicImage, GenericImage, ImageFormat };
use rocket::http::ContentType;
use rocket::response::Content;
use std::fs::File;
use std::io;
use std::io::{ BufReader, Write };
use std::path::PathBuf;
use regex::Regex;

#[derive(Debug)]
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

impl From<image::ImageError> for PadGridError {

    fn from(_: image::ImageError) -> PadGridError {
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

fn monster_icon(
    id: usize,
    handle: tokio_core::reactor::Handle
) -> impl Future<Item=DynamicImage, Error=PadGridError> {
    monster_icon_file(id, handle).and_then(move |file| {
        image::load(BufReader::new(file), ImageFormat::PNG).map_err(From::from)
    })
}

enum GridCell {
    Empty,
    Annotation(char),
    Icon(DynamicImage),
}

impl std::fmt::Debug for GridCell {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use GridCell::*;
        match *self {
            Empty => {
                write!(f, "[]")
            },
            Annotation(c) => {
                write!(f, "[{}]", c)
            },
            Icon(ref img) => {
                write!(f, "[{}x{}]", img.width(), img.height())
            }
        }
    }

}

fn cell_desc_to_cell(
    cell_desc: String,
    handle: tokio_core::reactor::Handle,
) -> impl Future<Item=GridCell, Error=PadGridError> {
    let fallback_cell = match cell_desc.len() {
        0 => { GridCell::Empty }
        1 => { GridCell::Annotation(cell_desc.chars().nth(0).unwrap()) },
        _ => { GridCell::Annotation('?') }
    };
    future::result(cell_desc.parse::<usize>()).and_then(move |id| {
        monster_icon(id, handle).map(move |image| {
            GridCell::Icon(image)
        }).or_else(move |_| {
            Ok(GridCell::Annotation('?'))
        })
    }).or_else(move |_| -> Result<GridCell, PadGridError> {
        Ok(fallback_cell)
    })
}

fn row_desc_to_cells(
    row_desc: Vec<String>,
    handle: tokio_core::reactor::Handle
) -> impl Future<Item=Vec<GridCell>, Error=PadGridError> {
    let handles = std::iter::repeat(handle);
    let cell_futures = row_desc.into_iter().zip(handles).map(move |(cell_desc, handle)| {
        cell_desc_to_cell(cell_desc, handle)
    });
    future::join_all(cell_futures)
}

fn grid_desc_to_cells(
    grid_desc: Vec<Vec<String>>,
    handle: tokio_core::reactor::Handle
) -> impl Future<Item=Vec<Vec<GridCell>>, Error=PadGridError> {
    let handles = std::iter::repeat(handle);
    let row_futures = grid_desc.into_iter().zip(handles).map(move |(row_desc, handle)| {
        row_desc_to_cells(row_desc, handle)
    });
    future::join_all(row_futures)
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
    let grid_desc = description.split(";").map(|row| {
        row.split(",").map(ToOwned::to_owned).collect::<Vec<String>>()
    }).collect::<Vec<Vec<String>>>();

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let grid = core.run(grid_desc_to_cells(grid_desc, handle)).unwrap();

    let rows = grid.len();
    let cols = grid.iter().map(|row| { row.len() }).max().unwrap_or(0);

    Some(format!("{}x{}: {:?}", cols, rows, grid))
}

fn main() {
    rocket::ignite().mount("/", routes![
        grid,
        monster
    ]).launch();
}
