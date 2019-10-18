extern crate osmpbfreader;

use osmpbfreader::{groups, primitive_block_from_blob};
use std::collections::HashMap;

fn parse_speed(max_speed: String, highway: String) -> u32 {
    let test = max_speed.trim().parse::<u32>();
    match test {
        Ok(ok) => return ok,
        Err(e) => {
            // println!("not a decimal ({:?}): {:?}", e, max_speed);
            // TODO parsing
            return aproximate_speed(highway);
        }
    }
}

fn aproximate_speed(s: String) -> u32 {
    match s.as_ref() {
        "motorway" => return 120,
        "motorway_link" => return 60,
        "trunk" => return 100,
        "trunk_link" => return 50,
        "primary" => return 60,
        "primary_link" => return 50,
        "secondary" | "secondary_link" => return 50,
        "tertiary" | "tertiary_link" => return 50,
        "unclassified" => return 40,
        "residential" => return 30,
        "service" => return 10,
        "living_street" => return 50,
        _ => return 50,
    }
}

fn distance(lat_1: f64, long_1: f64, lat_2: f64, long_2: f64) -> f64 {
    let r: f64 = 6371.0; // used for meters
    let d_lat: f64 = (lat_2 - lat_1).to_radians();
    let d_lon: f64 = (long_2 - long_1).to_radians();
    let lat1: f64 = (lat_1).to_radians();
    let lat2: f64 = (lat_2).to_radians();

    let a: f64 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
        + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
    let c: f64 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));
    return r * c;
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} pbf_file", args[0]);
        return;
    }

    let filename = std::env::args_os().nth(1).unwrap();
    let path = std::path::Path::new(&filename);
    let r = std::fs::File::open(&path).unwrap();
    let mut pbf = osmpbfreader::OsmPbfReader::new(r);

    // first store all way-IDs (in heap?) that are having the "highway" tag. also store speed-limit
    for obj in pbf.par_iter().map(Result::unwrap) {
        if obj.is_way() && obj.way().unwrap().tags.contains_key("highway") {
            let way = obj.way().unwrap();
            let highway = way.tags.get("highway").unwrap().to_string();
            let mut max_speed = "".to_string();
            if way.tags.contains_key("maxspeed") {
                max_speed = way.tags.get("maxspeed").unwrap().to_string();
            }
            let weight = parse_speed(max_speed, highway);
            println!("{:?} ### {:?}", way, weight);
            // get all node IDs from ways without duplication
            // for node in way.nodes {
            // have zip iterator over nodes from [0]:[1] to [n-1]:[n]
            // println!("!!!!{:?}", node);
            // }
        }
    }
    // reset pbf reader
    pbf.rewind();
    // store all geo-information about the nodes (also save min and max of long and lat)

    // calculate the time of all ways
    // create offset_table
    // serialize everything

    /*
    result of this program:
        int[] source, target, weight
        int[] offset_table (node id contains the index of source)
        double[] latitude, longitude
    */
}
