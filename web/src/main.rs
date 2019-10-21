extern crate actix_files;
extern crate actix_web;
extern crate bincode;
extern crate serde;
extern crate serde_json;

mod dijkstra;

use actix_files as fs;
use actix_web::{middleware, web, App, HttpServer};
use bincode::deserialize_from;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Input {
    target: Vec<u32>,
    weight: Vec<u32>,
    kind: Vec<u8>,
    latitude: Vec<f32>,
    longitude: Vec<f32>,
    offset_table: Vec<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Point {
    latitude: f32,
    longitude: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Query {
    start: Point,
    end: Point,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    path: Vec<Point>,
}

fn query(request: web::Json<Query>, input: web::Data<Input>) -> web::Json<Response> {
    // extract points
    let start: &Point = &request.start;
    let end: &Point = &request.end;
    println!("Start: {},{}", start.latitude, start.longitude);
    println!("End: {},{}", end.latitude, end.longitude);
    // search for clicked points
    let start_id: u32 = dijkstra::get_point_id(start.latitude, start.longitude, &input);
    let end_id: u32 = dijkstra::get_point_id(end.latitude, end.longitude, &input);
    println!("Point IDs: {},{}", start_id, end_id);
    let shortest_path: Vec<u32> = dijkstra::dijkstra(start_id, end_id, &input);
    for entry in shortest_path {
        print!("{} ", entry);
    }
    // TODO start dijkstra (pass by reference? and init distance with inifite)
    // TODO save vector of nodes
    // TODO convert vector to geo points
    // TMP begin
    let mut tmp_path: Vec<Point> = Vec::<Point>::new();
    tmp_path.push(Point {
        latitude: 11.11,
        longitude: 22.22,
    });
    tmp_path.push(Point {
        latitude: 33.33,
        longitude: 44.44,
    });
    // TMP end
    return web::Json(Response { path: tmp_path });
}

fn main() {
    // check if arguments are right
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} pbf.fmi_file", args[0]);
        return;
    }

    // check if file is right
    let filename = std::env::args_os().nth(1).unwrap();
    if !Path::new(&filename).exists() {
        println!("{} not found", filename.into_string().unwrap());
        std::process::exit(1);
    }

    // read file
    let mut f = BufReader::new(File::open(filename).unwrap());
    let input: Input = deserialize_from(&mut f).unwrap();
    let data = web::Data::new(input);

    // check for static-html folder
    if !Path::new("./static").exists() {
        eprintln!("./static/ directory not found");
        std::process::exit(1);
    }

    // start webserver
    println!("webserver started on http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(1024))
            .register_data(data.clone())
            .service(web::resource("/dijkstra").route(web::post().to(query)))
            .service(fs::Files::new("/", "./static/").index_file("index.html"))
    })
    .bind("localhost:8080")
    .unwrap()
    .run()
    .unwrap();
}
