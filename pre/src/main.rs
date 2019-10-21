extern crate bincode;
extern crate osmpbfreader;
extern crate serde;

use bincode::serialize_into;
use osmpbfreader::{groups, primitive_block_from_blob};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

// let get_type: () = var;

#[derive(Serialize, Debug)]
struct Output {
    target: Vec<u32>,
    weight: Vec<u32>,
    kind: Vec<u8>,
    latitude: Vec<f32>,
    longitude: Vec<f32>,
    offset_table: Vec<u32>,
}

fn parse_speed(max_speed: String, highway: String) -> u32 {
    match max_speed.trim().parse::<u32>() {
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
fn resolve_max_speed(s: String) -> Result<u32, String> {
    match s.to_ascii_lowercase().trim() {
        "de:rural" => return Ok(100),
        "at:rural" => return Ok(100),
        "at:urban" => return Ok(100),
        "de:urban" => return Ok(50),
        "30 kph" => return Ok(30),
        "zone:maxspeed=de:30" => return Ok(30),
        "de:zone:30" => return Ok(30),
        "50;" => return Ok(50),
        "50b" => return Ok(50),
        "10 mph" => return Ok(10),
        "de:living_street" => return Ok(30),
        "de:motorway" => return Ok(120),
        "5 mph" => return Ok(5),
        "maxspeed=50" => return Ok(50),
        "de:walk" => return Ok(3),
        "de:zone30" => return Ok(30),
        "cz:urban" => return Ok(30),
        "schrittgeschwindigkeit" => return Ok(3),
        "30 mph" => return Ok(30),
        "20:forward" => return Ok(20),
        "walk" => return Ok(3),
        _ => return Err("none".to_string()),
    };
}

/// approximates the speed limit based on given highway type
fn aproximate_speed_limit(s: String) -> u32 {
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
        "walk" => return 3,
        _ => return 50,
    }
}

fn main() {
    let mut source = Vec::<u32>::new();
    let mut target = Vec::<u32>::new();
    let mut weight = Vec::<u32>::new();
    let mut kind = Vec::<u8>::new();
    let mut latitude = Vec::<f32>::new();
    let mut longitude = Vec::<f32>::new();
    let mut offset_table = Vec::<u32>::new();

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
    let mut osm_id_mapping = HashMap::<i64, u32>::new();

    // first store all way-IDs (in heap?) that are having the "highway" tag. also store speed-limit
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for way in groups::ways(&group, &block) {
                if way.tags.contains_key("highway") {
                    let highway = way.tags.get("highway").unwrap().to_string();
                    let mut max_speed = "".to_string();
                    if way.tags.contains_key("maxspeed") {
                        max_speed = way.tags.get("maxspeed").unwrap().to_string();
                    }
                    let speed = parse_speed(max_speed, highway);
                    // get all node IDs from ways without duplication
                    let mut prev_id: u32;
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
                        source.push(prev_id);
                        target.push(id);
                        weight.push(speed);
                        kind.push(1);
                        prev_id = id;
                    }
                }
            }
        }
    }

    // resize offset_table, latitude, longitude based on amount_nodes
    latitude.resize(amount_nodes as usize, 0.0);
    longitude.resize(amount_nodes as usize, 0.0);
    offset_table.resize((amount_nodes + 1) as usize, 0);

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
                    let id = *osm_id_mapping.get(&osm_id).unwrap();
                    // then get geo infos and save
                    // TODO check if dividing could be improved
                    latitude[id as usize] = node.decimicro_lat as f32 / 10000000.0;
                    longitude[id as usize] = node.decimicro_lon as f32 / 10000000.0;
                }
            }
        }
    }

    let mut current_index = 0u32;
    for i in 0..source.len() {
        let s = source[i];
        // creat offset_table
        if s != current_index {
            offset_table[s as usize] = i as u32;
            current_index = s;
        }
    }
    // add additional last element for easier iterations later
    offset_table[source.len()] = source.len() as u32;
    // println!("{:?}", offset_table);

    // serialize everything
    let result = Output {
        target: target,
        weight: weight,
        kind: kind,
        latitude: latitude,
        longitude: longitude,
        offset_table: offset_table,
    };
    let output_file = format!("{}{}", filename.into_string().unwrap(), ".fmi");
    println!("everything gets written to {}", output_file);
    let mut f = BufWriter::new(File::create(output_file).unwrap());
    serialize_into(&mut f, &result).unwrap();
}
