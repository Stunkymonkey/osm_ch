extern crate bincode;
extern crate osmpbfreader;
extern crate serde;

mod constants;
mod osm_parsing;
mod structs;

#[cfg(test)]
mod tests;

use bincode::serialize_into;
use osmpbfreader::{groups, primitive_block_from_blob};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use constants::*;
use osm_parsing::*;
use structs::*;

/// fill offset array
fn fill_offset(edges: Vec<NodeId>, offset: &mut Vec<usize>) {
    for edge in edges {
        offset[edge + 1] += 1;
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

fn main() {
    let mut nodes = Vec::<Node>::new();
    let mut edges = Vec::<Way>::new();
    let mut up_offset = Vec::<usize>::new();
    let mut down_offset = Vec::<usize>::new();
    let mut grid = Vec::<usize>::new();
    let mut grid_offset = Vec::<usize>::new();

    let mut amount_nodes = 0;

    // check if arguments are right
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} pbf-file", args[0]);
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
                    let mut has_sidewalk: bool = false;
                    if way.tags.contains_key("sidewalk") {
                        has_sidewalk = match way.tags.get("sidewalk").unwrap().trim() {
                            "None" | "none" | "No" | "no" => false,
                            _ => true,
                        }
                    }
                    let travel_type = get_street_type(highway, has_sidewalk);
                    if travel_type == 100 {
                        continue;
                    }
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
                        edges.push(Way {
                            source: prev_id,
                            target: id,
                            speed: speed,
                            distance: 0,
                            travel_type: travel_type,
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
    up_offset.resize(amount_nodes + 1, 0);
    down_offset.resize(amount_nodes + 1, 0);
    grid.resize(amount_nodes, 0);

    // reset pbf reader
    match pbf.rewind() {
        Ok(_ok) => (),
        Err(_e) => panic!("rewind was not successfull"),
    }

    let mut lat_min = std::f32::MAX;
    let mut lat_max = std::f32::MIN;
    let mut lng_min = std::f32::MAX;
    let mut lng_max = std::f32::MIN;

    // store all geo-information about nodes
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
                        };
                        // save min and max coordinates
                        if latitude < lat_min {
                            lat_min = latitude;
                        } else if lat_max < latitude {
                            lat_max = latitude;
                        }
                        if longitude < lng_min {
                            lng_min = longitude;
                        } else if lng_max < longitude {
                            lng_max = longitude;
                        }
                    }
                    None => continue,
                }
            }
        }
    }

    // generate grid
    let lat_range = lat_max - lat_min;
    let lng_range = lng_max - lng_min;

    let mut tmp_grid = vec![vec![0; 0]; LAT_GRID_AMOUNT * LNG_GRID_AMOUNT];

    for (i, node) in nodes.iter().enumerate() {
        let lat_percent = (node.latitude - lat_min) / lat_range;
        let lat_index = (lat_percent * (LAT_GRID_AMOUNT - 1) as f32) as usize;
        let lng_percent = (node.longitude - lng_min) / lng_range;
        let lng_index = (lng_percent * (LNG_GRID_AMOUNT - 1) as f32) as usize;
        let grid_index = lng_index * LAT_GRID_AMOUNT + lat_index;
        tmp_grid[grid_index].push(i);
    }

    // convert tmp_grid to real grid
    grid_offset.resize((LAT_GRID_AMOUNT * LNG_GRID_AMOUNT) + 1, 0);
    let mut k = 0;
    for (i, cell) in tmp_grid.iter().enumerate() {
        grid.extend(cell.iter().cloned());
        grid_offset[i] = k;
        k += cell.len();
    }

    // generate up edges
    edges.sort_by(|a, b| a.source.cmp(&b.source));
    let sources: Vec<EdgeId> = edges.iter().map(|x| x.source).rev().collect();
    fill_offset(sources, &mut up_offset);

    // generate down edges, but without sorting edges
    let targets: Vec<EdgeId> = edges.iter().map(|x| x.target).rev().collect();
    fill_offset(targets, &mut down_offset);
    let mut down_index = vec![0; edges.len()];
    for (i, edge) in edges.iter().enumerate() {
        let start_index = down_offset[edge.target];
        let end_index = down_offset[edge.target + 1];
        for j in start_index..end_index {
            if down_index[j] == 0 {
                down_index[j] = i;
            }
        }
    }

    /*
    TODO write test
    for i in 0..100 {
        println!(
            "same {:?}",
            down_index[down_offset[edges[up_offset[i]].target]] == up_offset[i]
        );
    }
    */

    // contract edges
    // ordering
    let mut edge_distance = vec![0; amount_nodes];
    let mut contracted_neighbors = vec![0; amount_nodes];
    // TODO

    // serialize everything
    let result = FmiFile {
        nodes,
        edges,
        up_offset,
        down_index,
        down_offset,
        grid,
        grid_offset,
    };

    let output_file = format!("{}{}", filename.into_string().unwrap(), ".fmi");
    println!("everything gets written to {}", output_file);
    let mut writer = BufWriter::new(File::create(output_file).unwrap());
    serialize_into(&mut writer, &result).unwrap();

    /* new pipeline:
    read file
    get edges
    get nodes
    sort edges
    generate grid
    bidirectional edges
    contraction hierarchies
    export
    */
}
