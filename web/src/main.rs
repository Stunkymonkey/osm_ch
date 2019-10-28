extern crate actix_files;
extern crate actix_web;
extern crate bincode;
extern crate serde;
extern crate serde_json;

mod graph;

use actix_files as fs;
use actix_web::{middleware, web, App, HttpServer};
use bincode::deserialize_from;
use graph::Graph;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Way {
    source: usize,
    target: usize,
    weight: usize,
    kind: usize,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct Node {
    latitude: f32,
    longitude: f32,
}

#[derive(Deserialize, Debug)]
struct Input {
    nodes: Vec<Node>,
    ways: Vec<Way>,
    offset: Vec<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Query {
    start: Node,
    end: Node,
    use_car: bool,
    by_distance: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    path: Vec<Node>,
}

fn query(request: web::Json<Query>, dijkstra: web::Data<Graph>) -> web::Json<Response> {
    let total_time = Instant::now();
    // extract points
    let start: &Node = &request.start;
    let end: &Node = &request.end;
    let use_car: bool = request.use_car;
    let by_distance: bool = request.by_distance;
    // println!("Start: {},{}", start.latitude, start.longitude);
    // println!("End: {},{}", end.latitude, end.longitude);
    // println!("use_car: {}, by_distance: {}", use_car, by_distance);

    let timing = Instant::now();

    // search for clicked points
    let start_id: usize = dijkstra.get_point_id(start.latitude, start.longitude);
    let end_id: usize = dijkstra.get_point_id(end.latitude, end.longitude);

    println!("### duration for get_point_id(): {:?}", timing.elapsed());

    let timing = Instant::now();

    let tmp = dijkstra.find_path(start_id, end_id, use_car, by_distance);
    println!("### duration for find_path(): {:?}", timing.elapsed());

    let result: Vec<Node>;
    match tmp {
        Some((path, _cost)) => {
            let timing = Instant::now();

            result = dijkstra.get_coordinates(path);

            println!("### duration for get_coordinates(): {:?}", timing.elapsed());
        }
        None => {
            println!("no path found");
            result = Vec::<Node>::new();
        }
    }
    println!("### answered request in: {:?}", total_time.elapsed());

    return web::Json(Response { path: result });
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
    let d = Graph::new(input.nodes, input.ways, input.offset);

    let graph = web::Data::new(d);

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
            .register_data(graph.clone())
            .service(web::resource("/dijkstra").route(web::post().to(query)))
            .service(fs::Files::new("/", "./static/").index_file("index.html"))
    })
    .bind("localhost:8080")
    .unwrap()
    .run()
    .unwrap();
}
