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
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Way {
    source: usize,
    target: usize,
    speed: usize,
    distance: usize,
    travel_type: usize,
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
    grid: HashMap<(usize, usize), Vec<usize>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Query {
    start: Node,
    end: Node,
    travel_type: String,
    by_distance: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    path: Vec<Node>,
    cost: String,
}

fn query(request: web::Json<Query>, dijkstra: web::Data<Graph>) -> web::Json<Response> {
    let total_time = Instant::now();
    // extract points
    let start: &Node = &request.start;
    let end: &Node = &request.end;
    let travel_type = match request.travel_type.as_ref() {
        "car" => 0,
        "bicycle" => 1,
        "foot" => 2,
        _ => 0,
    };
    let by_distance: bool = request.by_distance;
    // println!("Start: {},{}", start.latitude, start.longitude);
    // println!("End: {},{}", end.latitude, end.longitude);
    // println!("travel_type: {}, by_distance: {}", travel_type, by_distance);

    // search for clicked points
    let timing_find = Instant::now();
    let start_id: usize = dijkstra.get_point_id(start.latitude, start.longitude, travel_type);
    let end_id: usize = dijkstra.get_point_id(end.latitude, end.longitude, travel_type);
    println!(
        "### duration for get_point_id(): {:?}",
        timing_find.elapsed()
    );

    let timing = Instant::now();
    let tmp = dijkstra.find_path(start_id, end_id, travel_type, by_distance);
    println!("### duration for find_path(): {:?}", timing.elapsed());

    let result: Vec<Node>;
    let mut cost: String = "".to_string();
    match tmp {
        Some((path, path_cost)) => {
            result = dijkstra.get_coordinates(path);
            match by_distance {
                false => {
                    if path_cost.trunc() >= 1.0 {
                        cost = path_cost.trunc().to_string();
                        cost.push_str("h ");
                    }
                    cost.push_str(&format!("{:.0}", path_cost.fract() * 60.0));
                    cost.push_str("min");
                }
                true => {
                    cost = format!("{:.2}", path_cost);
                    cost.push_str("km");
                }
            };
        }
        None => {
            println!("no path found");
            result = Vec::<Node>::new();
            cost = 0.to_string();
        }
    }

    println!("### answered request in: {:?}", total_time.elapsed());

    return web::Json(Response {
        path: result,
        cost: cost,
    });
}

fn main() {
    // check if arguments are right
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} pbf.fmi-file", args[0]);
        return;
    }

    // check if file is right
    let filename = std::env::args_os().nth(1).unwrap();
    if !Path::new(&filename).exists() {
        println!("{} not found", filename.into_string().unwrap());
        std::process::exit(1);
    }

    // read file
    let mut reader = BufReader::new(File::open(filename).unwrap());
    let input: Input = deserialize_from(&mut reader).unwrap();
    let d = Graph::new(input.nodes, input.ways, input.offset, input.grid);

    let graph = web::Data::new(d);

    // check for static-html folder
    if !Path::new("./html").exists() {
        eprintln!("./html/ directory not found");
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
