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

fn parse_speed(max_speed: &str, highway: &str) -> u32 {
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
fn resolve_max_speed(s: &str) -> Result<u32, &str> {
    match s {
        "DE:rural"|"AT:rural" => return Ok(100),
        "DE:urban"|"AT:urban"|"CZ:urban" => return Ok(50),
        "DE:walk"|"walk"|"Schrittgeschwindigkeit" => return Ok(7),
        "DE:living_street" => return Ok(30),
        "DE:motorway" => return Ok(120),
        "30 kph" => return Ok(30),
        "zone:maxspeed=de:30" => return Ok(30),
        "DE:zone:30" => return Ok(30),
        "50;"|"50b" => return Ok(50),
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
fn aproximate_speed_limit(s: &str) -> u32 {
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
                    let highway = way.tags.get("highway").unwrap().trim();
                    let mut max_speed: &str = "";
                    if way.tags.contains_key("maxspeed") {
                        max_speed = way.tags.get("maxspeed").unwrap().trim();
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
    latitude.resize((amount_nodes) as usize, 0.0);
    longitude.resize((amount_nodes) as usize, 0.0);
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

    fill_offset_table(&source, &mut offset_table);

    // add additional last element for easier iterations later
    offset_table[amount_nodes as usize] = (source.len() - 1) as u32;
    // println!("{:?}", offset_table);

    assert_eq!(offset_table.len(), (amount_nodes + 1) as usize);
    assert_eq!(source.len(), target.len());
    assert_eq!(source.len(), weight.len());
    assert_eq!(latitude.len(), longitude.len());
    assert_eq!(latitude.len(), amount_nodes as usize);

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

fn fill_offset_table(sources: &Vec<u32>, mut offset_table: &mut Vec<u32>) {
    // initialize index 0 (if 0 is not the first node in sources this is still correct)
    offset_table[0] = 0;

    let mut last_updated_node= 0;
    // fill rest of offset_table (from index 1)
    let mut i: u32 = 0;
    for node in sources.iter() {
        if node > &last_updated_node {
            // update all nodes that were not contained in source up to the current node_id
            for j in (last_updated_node+1)..(node + 1) {
                offset_table[j as usize] = i;
            }
            last_updated_node = *node;
        }
        i+=1;
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_fill_offset_table() {
        let mut offset_test = vec![0; 7];
        let sources: Vec<u32> = vec![0,0,0,2,3,4,4,4,6];

        fill_offset_table(&sources, &mut offset_test);

        //1 is not a valid node
        assert_eq!(offset_test[0], 0);
        assert_eq!(offset_test[1], 3);
        assert_eq!(offset_test[2], 3);
        assert_eq!(offset_test[3], 4);
        assert_eq!(offset_test[4], 5);
        assert_eq!(offset_test[5], 8);
        assert_eq!(offset_test[6], 8);
    }
}
