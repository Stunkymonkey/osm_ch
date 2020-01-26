extern crate actix_files;
extern crate actix_rt;
extern crate actix_web;
extern crate bincode;
extern crate serde;
extern crate rayon;
extern crate serde_json;

mod constants;
mod dijkstra;
mod grid;
mod helper;
mod min_heap;
mod structs;
mod visited_list;

use rayon::prelude::*;
use actix_web::{middleware, web, App, HttpServer};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

use constants::*;
use dijkstra::Dijkstra;
use structs::*;

async fn query(
    request: web::Json<Query>,
    data: web::Data<FmiFile>,
    dijkstra: web::Data<Dijkstra>,
) -> web::Json<Response> {
    let total_time = Instant::now();
    // extract points
    let start: Node = request.start;
    let end: Node = request.end;
    // println!("Start: {},{}", start.latitude, start.longitude);
    // println!("End: {},{}", end.latitude, end.longitude);

    // search for clicked points
    let grid_time = Instant::now();
    let start_id: NodeId = grid::get_closest_point(start, &data.nodes, &data.grid_bounds);
    let end_id: NodeId = grid::get_closest_point(end, &data.nodes, &data.grid_bounds);
    println!("Getting node IDs in: {:?}", grid_time.elapsed());

    let dijkstra_time = Instant::now();
    // let tmp = dijkstra.find_path(start_id, end_id);
    let cost: f32 = 1.2;
    let tmp = Some((vec![400, 300, 200, 100, 500, 600], cost));
    println!("Getting path in: {:?}", dijkstra_time.elapsed());

    let result: Vec<(f32, f32)>;
    let mut cost: String = "".to_string();
    match tmp {
        Some((path, path_cost)) => {
            let nodes = grid::get_coordinates(path, &data.nodes);
            result = nodes.par_iter().map(|node| (node.longitude, node.latitude)).collect::<Vec<(f32, f32)>>();
            match data.optimized_by {
                OptimizeBy::Time => {
                    if path_cost.trunc() >= 1.0 {
                        cost = path_cost.trunc().to_string();
                        cost.push_str("h ");
                    }
                    cost.push_str(&format!("{:.0}", path_cost.fract() * 60.0));
                    cost.push_str("min");
                }
                OptimizeBy::Distance => {
                    cost = format!("{:.2}", path_cost);
                    cost.push_str("km");
                }
            };
        }
        None => {
            println!("no path found");
            result = Vec::<(f32, f32)>::new();
            cost = 0.to_string();
        }
    }

    println!("Overall: {:?}", total_time.elapsed());

    return web::Json(Response {
        // escaping the rust-type to normal type string
        r#type: "LineString".to_string(),
        coordinates: result,
        properties: ResponseWeight { weight: cost },
    });
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // read file
    let filename = helper::get_filename();
    let data: FmiFile = helper::read_from_disk(filename);

    // initialize dijkstra
    let dijkstra: Dijkstra = Dijkstra::new(data.nodes.len());

    let data_ref = web::Data::new(data);

    // check for static-html folder
    if !Path::new("./html").exists() {
        eprintln!("./html/ directory not found");
        std::process::exit(1);
    }

    // start webserver
    println!("Starting server at: http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(1024))
            .data(dijkstra.clone())
            .app_data(data_ref.clone())
            .service(web::resource("/dijkstra").route(web::post().to(query)))
            .service(actix_files::Files::new("/", "./html/").index_file("index.html"))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
