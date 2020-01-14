use super::*;
use osmpbfreader::{groups, primitive_block_from_blob, OsmPbfReader};
use std::fs::File;
use std::path::Path;

pub fn get_pbf(filename: &String) -> osmpbfreader::OsmPbfReader<std::fs::File> {
    let path = Path::new(&filename);
    if !path.exists() {
        println!("{} not found", filename);
        std::process::exit(1);
    }
    let r = File::open(&path).unwrap();
    return OsmPbfReader::new(r);
}

/// store all way-IDs that are having the "highway" tag. with speed-limit
pub fn read_edges(
    pbf: &mut osmpbfreader::OsmPbfReader<std::fs::File>,
    full_edges: &mut Vec<FullWay>,
    osm_id_mapping: &mut HashMap<i64, usize>,
) {
    let mut amount_nodes = 0;
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for way in groups::ways(&group, &block) {
                if way.tags.contains_key("highway") {
                    let highway = way.tags.get("highway").unwrap().trim();
                    let mut has_sidewalk: bool = false;
                    if way.tags.contains_key("sidewalk") {
                        has_sidewalk = match way.tags.get("sidewalk").unwrap().trim() {
                            "None" | "none" | "No" | "no" => false,
                            _ => true,
                        }
                    }
                    let travel_type = osm_parsing::get_street_type(highway, has_sidewalk);
                    if !is_sub_travel_type(travel_type) {
                        continue;
                    }
                    let mut max_speed: &str = "";
                    if way.tags.contains_key("maxspeed") {
                        max_speed = way.tags.get("maxspeed").unwrap().trim();
                    }
                    let speed = osm_parsing::parse_speed(max_speed, highway);
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
                        full_edges.push(FullWay {
                            source: prev_id,
                            target: id,
                            speed: speed,
                            distance: 0,
                        });
                        prev_id = id;
                    }
                }
            }
        }
    }
}

/// store all geo-information about nodes
pub fn read_ways(
    pbf: &mut osmpbfreader::OsmPbfReader<std::fs::File>,
    nodes: &mut Vec<Node>,
    osm_id_mapping: &mut HashMap<i64, usize>,
) {
    // reset pbf reader
    osm_pbf::reset_pbf(pbf);

    // resize nodes
    nodes.resize(
        osm_id_mapping.len(),
        Node {
            latitude: 0.,
            longitude: 0.,
            rank: INVALID_RANK,
        },
    );
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
                            latitude,
                            longitude,
                            rank: INVALID_RANK,
                        };
                    }
                    None => continue,
                }
            }
        }
    }
}

/// rewinds the reader back to the beginning of the file
pub fn reset_pbf(pbf: &mut osmpbfreader::OsmPbfReader<std::fs::File>) {
    match pbf.rewind() {
        Ok(_ok) => (),
        Err(_e) => panic!("rewind was not successfull"),
    }
}

/// check if the travel type matches the given one
pub fn is_sub_travel_type(travel_type: TravelType) -> bool {
    match TRAVEL_TYPE {
        TravelType::Car => match travel_type {
            TravelType::Car | TravelType::CarBicycle | TravelType::All => return true,
            _ => return false,
        },
        TravelType::CarBicycle => match travel_type {
            TravelType::Car
            | TravelType::CarBicycle
            | TravelType::Bicycle
            | TravelType::BicyclePedestrian
            | TravelType::All => return true,
            _ => return false,
        },
        TravelType::Bicycle => match travel_type {
            TravelType::CarBicycle
            | TravelType::Bicycle
            | TravelType::BicyclePedestrian
            | TravelType::All => return true,
            _ => return false,
        },
        TravelType::BicyclePedestrian => match travel_type {
            TravelType::CarBicycle
            | TravelType::Bicycle
            | TravelType::BicyclePedestrian
            | TravelType::Pedestrian
            | TravelType::All => return true,
            _ => return false,
        },
        TravelType::Pedestrian => match travel_type {
            TravelType::BicyclePedestrian | TravelType::Pedestrian | TravelType::All => {
                return true
            }
            _ => return false,
        },
        TravelType::All => match travel_type {
            TravelType::Undefined => return false,
            _ => return true,
        },
        TravelType::Undefined | _ => panic!("Senseless TravelType is set"),
    }
}
