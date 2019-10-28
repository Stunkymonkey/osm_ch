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

// let get_type: () = var;

#[derive(Serialize, Debug)]
struct Way {
    source: usize,
    target: usize,
    weight: usize,
    kind: usize,
}

#[derive(Serialize, Debug, Clone)]
struct Node {
    latitude: f32,
    longitude: f32,
}

#[derive(Serialize, Debug)]
struct Output {
    ways: Vec<Way>,
    nodes: Vec<Node>,
    offset: Vec<usize>,
}

fn parse_speed(max_speed: &str, highway: &str) -> usize {
    match max_speed.trim().parse::<usize>() {
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
                            weight: speed,
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
                let osm_id = node.id.0;
                // check if node in osm_id_mapping
                if osm_id_mapping.contains_key(&osm_id) {
                    // let id = *osm_id_mapping.get(&osm_id).unwrap();
                    // then get geo infos and save
                    nodes.push(Node {
                        latitude: node.decimicro_lat as f32 / 10000000.0,
                        longitude: node.decimicro_lon as f32 / 10000000.0,
                    });
                }
            }
        }
    }

    ways.sort_by(|a, b| b.source.cmp(&a.source));
    fill_offset(&ways, &mut offset);

    // serialize everything
    let result = Output {
        ways: ways,
        nodes: nodes,
        offset: offset,
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
