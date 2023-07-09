use super::*;
use osmpbfreader::{groups, primitive_block_from_blob, OsmPbfReader};
use std::collections::hash_map::Entry;
use std::fs::File;
use std::path::Path;

pub fn get_pbf(filename: &str) -> osmpbfreader::OsmPbfReader<std::fs::File> {
    let path = Path::new(&filename);
    if !path.exists() {
        println!("{} not found", filename);
        std::process::exit(1);
    }
    let r = File::open(path).unwrap();
    OsmPbfReader::new(r)
}

/// store all way-IDs that are having the "highway" tag. with speed-limit
pub fn read_edges(
    pbf: &mut osmpbfreader::OsmPbfReader<std::fs::File>,
    full_edges: &mut Vec<OsmWay>,
    osm_id_mapping: &mut HashMap<i64, usize>,
) {
    let mut amount_nodes = 0;
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for way in groups::ways(group, &block) {
                if way.tags.contains_key("highway") {
                    let highway = way.tags.get("highway").unwrap().trim();
                    let mut has_sidewalk: bool = false;
                    if way.tags.contains_key("sidewalk") {
                        has_sidewalk = !matches!(
                            way.tags.get("sidewalk").unwrap().trim(),
                            "None" | "none" | "No" | "no"
                        )
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
                    let mut one_way: &str = "";
                    if way.tags.contains_key("oneway") {
                        one_way = way.tags.get("oneway").unwrap().trim();
                    }
                    let (one_way, reverse_dir): (bool, bool) = osm_parsing::parse_one_way(one_way);
                    // get all node IDs from ways without duplication
                    let mut prev_id: usize;
                    let osm_id = way.nodes[0].0;
                    prev_id = match osm_id_mapping.entry(osm_id) {
                        Entry::Occupied(o) => *o.into_mut(),
                        Entry::Vacant(v) => {
                            amount_nodes += 1;
                            *v.insert(amount_nodes - 1)
                        }
                    };
                    // iterate over nodes and add them
                    for node in way.nodes.iter().skip(1) {
                        let osm_id = node.0;
                        let id = match osm_id_mapping.entry(osm_id) {
                            Entry::Occupied(o) => *o.into_mut(),
                            Entry::Vacant(v) => {
                                amount_nodes += 1;
                                *v.insert(amount_nodes - 1)
                            }
                        };
                        if !reverse_dir || !one_way {
                            full_edges.push(OsmWay {
                                source: prev_id,
                                target: id,
                                speed,
                                distance: 0,
                            });
                        }
                        if reverse_dir || !one_way {
                            full_edges.push(OsmWay {
                                source: id,
                                target: prev_id,
                                speed,
                                distance: 0,
                            });
                        }
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
            for node in groups::dense_nodes(group, &block) {
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
        TravelType::Car => matches!(
            travel_type,
            TravelType::Car | TravelType::CarBicycle | TravelType::All
        ),
        TravelType::Bicycle => matches!(
            travel_type,
            TravelType::CarBicycle
                | TravelType::Bicycle
                | TravelType::BicyclePedestrian
                | TravelType::All
        ),
        TravelType::Pedestrian => matches!(
            travel_type,
            TravelType::BicyclePedestrian | TravelType::Pedestrian | TravelType::All
        ),
        TravelType::All => true,
        _ => panic!("Invalid TravelType is set"),
    }
}
