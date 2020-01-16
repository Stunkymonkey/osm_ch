use super::*;
use bincode::serialize_into;
use std::fs::File;
use std::io::BufWriter;

pub fn get_filename() -> String {
    // check if arguments are right
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} pbf-file", args[0]);
        std::process::exit(1);
    }

    // get filename
    return std::env::args_os().nth(1).unwrap().into_string().unwrap();
}

pub fn write_to_disk(filename: String, result: FmiFile) -> String {
    let output_file = format!("{}{}", filename, ".fmi");
    let mut writer = BufWriter::new(File::create(&output_file).unwrap());
    serialize_into(&mut writer, &result).unwrap();
    return output_file;
}

/// get distance on earth surface using haversine formula
fn calc_distance(lat_1: f32, long_1: f32, lat_2: f32, long_2: f32) -> f32 {
    let r: f32 = 6371.0; // constant used for meters
    let d_lat: f32 = (lat_2 - lat_1).to_radians();
    let d_lon: f32 = (long_2 - long_1).to_radians();
    let lat1: f32 = (lat_1).to_radians();
    let lat2: f32 = (lat_2).to_radians();

    let a: f32 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
        + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
    let c: f32 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));
    return r * c;
}

// calculate edge distances
pub fn calc_edge_distances(full_edges: &mut Vec<FullWay>, nodes: &Vec<Node>) {
    full_edges.par_iter_mut().for_each(|edge| {
        edge.distance = (calc_distance(
            nodes[edge.source].latitude,
            nodes[edge.source].longitude,
            nodes[edge.target].latitude,
            nodes[edge.target].longitude,
        ) * DIST_MULTIPLICATOR as f32) as usize
    });
}

pub fn edges_to_weight(edges: &mut Vec<Way>, full_edges: &Vec<FullWay>) {
    for full_edge in full_edges {
        let speed = match TRAVEL_TYPE {
            TravelType::Car => full_edge.speed,
            TravelType::CarBicycle => full_edge.speed,
            TravelType::Bicycle if full_edge.speed <= 20 => full_edge.speed,
            TravelType::Bicycle => 20,
            TravelType::BicyclePedestrian if full_edge.speed <= 20 => full_edge.speed,
            TravelType::BicyclePedestrian => 20,
            TravelType::Pedestrian => 7,
            TravelType::All => full_edge.speed,
            TravelType::Undefined => 1,
        };
        let weight = match OPTIMIZE_BY {
            OptimizeBy::Distance => full_edge.distance,
            OptimizeBy::Time => full_edge.distance / speed,
        };
        edges.push(Way::new(full_edge.source, full_edge.target, weight));
    }
}
