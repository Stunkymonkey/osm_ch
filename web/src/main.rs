extern crate actix_web;
extern crate bincode;
extern crate serde;

use actix_web::{web, App, HttpServer, Responder};
use bincode::deserialize_from;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize, Debug)]
struct Input {
    source: Vec<u32>,
    target: Vec<u32>,
    weight: Vec<u32>,
    latitude: Vec<f32>,
    longitude: Vec<f32>,
    offset_table: Vec<u32>,
}

fn index(info: web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", info.1, info.0)
}

fn main() -> std::io::Result<()> {
    let filename = std::env::args_os().nth(1).unwrap();
    // todo check for file
    let mut f = BufReader::new(File::open(filename).unwrap());
    let input: Input = deserialize_from(&mut f).unwrap();

    println!("{:?}", input.source.len());

    HttpServer::new(|| App::new().service(web::resource("/{id}/{name}/index.html").to(index)))
        .bind("127.0.0.1:8080")?
        .run()
}
