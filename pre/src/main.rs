extern crate bincode;
extern crate osmpbfreader;
extern crate serde;

#[cfg(test)]
mod tests;

use bincode::serialize_into;
use osmpbfreader::{groups, primitive_block_from_blob};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

// 2^18
const COST_MULTIPLICATOR: usize = 262144;
// First three digits of coordinates are used for the grid hashing
const GRID_MULTIPLICATOR: usize = 100;

#[derive(Serialize, Debug, Clone)]
struct Way {
    source: usize,
    target: usize,
    speed: usize,
    distance: usize,
    kind: usize,
}

#[derive(Serialize, Debug, Clone)]
struct Node {
    latitude: f32,
    longitude: f32,
}

#[derive(Serialize, Debug)]
struct Output {
    nodes: Vec<Node>,
    ways: Vec<Way>,
    offset: Vec<usize>,
    grid: HashMap<(usize, usize), Vec<usize>>
}

fn parse_speed(max_speed: &str, highway: &str) -> usize {
    match max_speed.parse::<usize>() {
        Ok(ok) => return ok,
        Err(_e) => match resolve_max_speed(max_speed) {
            Ok(ok) => return ok,
            Err(_e) => {
                return aproximate_speed_limit(highway);
            }
        },
    }
}

/// resolves the int value from a dirty string that can't be resolved by default parsing
fn resolve_max_speed(s: &str) -> Result<usize, &str> {
    match s {
        "DE:rural" | "AT:rural" => return Ok(100),
        "DE:urban" | "AT:urban" | "CZ:urban" => return Ok(50),
        "DE:walk" | "walk" | "Schrittgeschwindigkeit" => return Ok(7),
        "DE:living_street" => return Ok(30),
        "DE:motorway" => return Ok(120),
        "30 kph" => return Ok(30),
        "zone:maxspeed=de:30" => return Ok(30),
        "DE:zone:30" => return Ok(30),
        "50;" | "50b" => return Ok(50),
        "10 mph" => return Ok(10),
        "5 mph" => return Ok(7),
        "maxspeed=50" => return Ok(50),
        "DE:zone30" => return Ok(30),
        "30 mph" => return Ok(30),
        "20:forward" => return Ok(20),
        _ => return Err("none"),
    };
}

/// approximates the speed limit based on given highway type
fn aproximate_speed_limit(s: &str) -> usize {
    match s {
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
        "walk" => return 3,
        _ => return 50,
    }
}

fn main() {
    let mut ways = Vec::<Way>::new();
    let mut nodes = Vec::<Node>::new();
    let mut offset = Vec::<usize>::new();
    // stores node ids for a 2d grid e.g. (1,1) = [1,2,3,..]
    let mut grid = HashMap::<(usize, usize), Vec<usize>>::new();

    let mut amount_nodes = 0;

    // check if arguments are right
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} pbf_file", args[0]);
        return;
    }

    // read pbf file
    let filename = std::env::args_os().nth(1).unwrap();
    let path = Path::new(&filename);
    if !path.exists() {
        println!("{} not found", filename.into_string().unwrap());
        std::process::exit(1);
    }
    let r = File::open(&path).unwrap();
    let mut pbf = osmpbfreader::OsmPbfReader::new(r);

    // for storing mapping of own-ids and osm-ids
    let mut osm_id_mapping = HashMap::<i64, usize>::new();

    // first store all way-IDs that are having the "highway" tag. also store speed-limit
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for way in groups::ways(&group, &block) {
                if way.tags.contains_key("highway") {
                    let highway = way.tags.get("highway").unwrap().trim();
                    let mut max_speed: &str = "";
                    if way.tags.contains_key("maxspeed") {
                        max_speed = way.tags.get("maxspeed").unwrap().trim();
                    }
                    let speed = parse_speed(max_speed, highway);
                    // get all node IDs from ways without duplication
                    let mut prev_id: usize;
                    let osm_id = way.nodes[0].0;
                    if osm_id_mapping.contains_key(&osm_id) {
                        prev_id = *osm_id_mapping.get(&osm_id).unwrap();
                    } else {
                        osm_id_mapping.insert(osm_id, amount_nodes);
                        prev_id = amount_nodes;
                        amount_nodes += 1;
                    }
                    // iterate over nodes and add them
                    for node in way.nodes.iter().skip(1) {
                        let osm_id = node.0;
                        let id;
                        if osm_id_mapping.contains_key(&osm_id) {
                            id = *osm_id_mapping.get(&osm_id).unwrap();
                        } else {
                            osm_id_mapping.insert(osm_id, amount_nodes);
                            id = amount_nodes;
                            amount_nodes += 1;
                        }
                        ways.push(Way {
                            source: prev_id,
                            target: id,
                            speed: speed,
                            distance: 0,
                            // TODO add what kind of street it is
                            kind: 1,
                        });
                        prev_id = id;
                    }
                }
            }
        }
    }

    // resize offset and nodes
    nodes.resize(
        amount_nodes,
        Node {
            latitude: 0.0,
            longitude: 0.0,
        },
    );
    offset.resize(amount_nodes + 1, 0);

    // reset pbf reader
    match pbf.rewind() {
        Ok(_ok) => (),
        Err(_e) => panic!("rewind was not successfull"),
    }

    // store all geo-information about the nodes
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for node in groups::dense_nodes(&group, &block) {
                // check if node in osm_id_mapping
                match osm_id_mapping.get(&node.id.0) {
                    Some(our_id) => {
                        let latitude = node.decimicro_lat as f32 / 10_000_000.0;
                        let longitude = node.decimicro_lon as f32 / 10_000_000.0;
                        nodes[*our_id] = Node {
                            // https://github.com/rust-lang/rfcs/blob/master/text/1682-field-init-shorthand.md
                            latitude,
                            longitude,
                        };
                        let lat_grid = (latitude * GRID_MULTIPLICATOR as f32) as i32;
                        let lng_grid = (longitude * GRID_MULTIPLICATOR as f32) as i32;
                        let current_grid = grid.get_mut(&(lat_grid as usize, lng_grid as usize));
                        match current_grid {
                            Some(id_list) => {
                                id_list.push(*our_id);
                            },
                            None => {
                                let mut new_id_list = Vec::<usize>::new();
                                new_id_list.push(*our_id);
                                grid.insert((lat_grid as usize, lng_grid as usize), new_id_list);
                            }
                        }
                    }
                    None => continue,
                }
            }
        }
    }

    ways.sort_by(|a, b| a.source.cmp(&b.source));
    fill_offset(&ways, &mut offset);

    let mut counter: usize = 0;
    let mut longest_way: f32 = 0.0;
    let mut shortest_way: f32 = 200.0;

    for i in 0..ways.len() {
        let distance = calc_distance(
            nodes[ways[i].source].latitude,
            nodes[ways[i].source].longitude,
            nodes[ways[i].target].latitude,
            nodes[ways[i].target].longitude,
        );
        ways[i].distance = (distance * COST_MULTIPLICATOR as f32) as usize;
        if ways[i].distance == 0 {
            counter += 1;
        }
        if distance >= longest_way {
            longest_way = distance;
        }
        if distance <= shortest_way {
            shortest_way = distance;
        }
    }
    println!("zero counter {:?}", counter);
    println!("long counter {:?}", longest_way);
    println!("short counter {:?}", shortest_way);

    // serialize everything
    let result = Output {
        nodes,
        ways,
        offset,
        grid
    };

    let output_file = format!("{}{}", filename.into_string().unwrap(), ".fmi");
    println!("everything gets written to {}", output_file);
    let mut f = BufWriter::new(File::create(output_file).unwrap());
    serialize_into(&mut f, &result).unwrap();
}

/// fill offset array
fn fill_offset(ways: &Vec<Way>, offset: &mut Vec<usize>) {
    for way in ways {
        offset[way.source + 1] += 1;
    }
    for i in 1..offset.len() {
        offset[i] += offset[i - 1];
    }
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
