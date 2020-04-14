#[macro_use]
extern crate log;

mod constants;
mod bidijkstra;
mod geojson;
mod graph_helper;
mod grid;
mod helper;
mod min_heap;
mod structs;
mod visited_list;

use rayon::prelude::*;
use actix_web::{middleware, web, App, HttpServer};
use std::cell::RefCell;
use std::path::Path;
use std::time::Instant;

use constants::*;
use bidijkstra::Dijkstra;
use structs::*;
use geojson::*;

async fn shortest_path(
    request: web::Json<GeoJsonRequest>,
    data: web::Data<FmiFile>,
    dijkstra_cell: web::Data<RefCell<Dijkstra>>,
) -> web::Json<GeoJsonRespone> {
    let total_time = Instant::now();

    // extract points
    let features = &request.features;
    assert_eq!(features.len(), 2);

    let start_feature = &features[0].geometry.coordinates;
    let end_feature = &features[1].geometry.coordinates;
    assert_eq!(start_feature.len(), 2);
    assert_eq!(end_feature.len(), 2);

    let start = Node { longitude : start_feature[0], latitude: start_feature[1], rank: INVALID_RANK};
    let end = Node { longitude : end_feature[0], latitude: end_feature[1], rank: INVALID_RANK};
    debug!("Start: {},{}", start.latitude, start.longitude);
    debug!("End: {},{}", end.latitude, end.longitude);

    // search for clicked points
    let grid_time = Instant::now();
    let start_id: NodeId = grid::get_closest_point(start, &data.nodes, &data.grid, &data.grid_offset, &data.grid_bounds);
    let end_id: NodeId = grid::get_closest_point(end, &data.nodes, &data.grid, &data.grid_offset, &data.grid_bounds);
    debug!("start_id {}", start_id);
    debug!("end_id {}", end_id);
    info!(" Get node-ID in: {:?}", grid_time.elapsed());

    let mut dijkstra = dijkstra_cell.borrow_mut();

    let dijkstra_time = Instant::now();
    let tmp = dijkstra.find_path(start_id, end_id, &data.nodes, &data.edges, &data.up_offset, &data.down_offset, &data.down_index);
    info!("    Dijkstra in: {:?}", dijkstra_time.elapsed());

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
                        cost.push_str(" h ");
                    }
                    cost.push_str(&format!("{:.0}", path_cost.fract() * 60.0));
                    cost.push_str(" min");
                }
                OptimizeBy::Distance => {
                    cost = format!("{:.2}", path_cost);
                    cost.push_str(" km");
                }
            };
        }
        None => {
            warn!("no path found");
            result = Vec::<(f32, f32)>::new();
            cost = "no path found".to_string();
        }
    }

    info!("        Overall: {:?}", total_time.elapsed());

    return web::Json(GeoJsonRespone {
        // escaping the rust-type command to normal type string
        r#type: "FeatureCollection".to_string(),
        features: vec![ FeatureResponse { r#type: "Feature".to_string(), geometry: GeometryResponse{r#type: "LineString".to_string(), coordinates: result}, properties: Some(Property { weight: cost }) }],
    });
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // read file
    let filename = helper::get_filename();
    let data: FmiFile = helper::read_from_disk(filename);

    let amount_nodes = data.nodes.len();
    let data_ref = web::Data::new(data);

    // check for static-html folder
    if !Path::new("./html").exists() {
        eprintln!("./html/ directory not found");
        std::process::exit(1);
    }

    // start webserver
    println!("Starting server at: http://localhost:8080");
    HttpServer::new(move || {
        // initialize thread-local dijkstra
        let dijkstra = RefCell::new(Dijkstra::new(amount_nodes));
        App::new()
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(1024))
            .app_data(data_ref.clone())
            .data(dijkstra)
            .service(web::resource("/dijkstra").route(web::post().to(shortest_path)))
            .service(actix_files::Files::new("/", "./html/").index_file("index.html"))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
