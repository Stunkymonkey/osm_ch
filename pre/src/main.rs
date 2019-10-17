extern crate osmpbfreader;

use osmpbfreader::{groups, primitive_block_from_blob};

fn parse_maxspeed(s: String) -> u32 {
    let test = s.trim().parse::<u32>();
    match test {
        Ok(ok) => return ok,
        Err(e) => {
            println!("not a decimal ({:?}): {:?}", e, s);
        }
    }
    return 0;
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
    // first store all way-IDs (in binary heap?) that are having the "highway" tag. also store speed-limit
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for way in groups::ways(&group, &block) {
                if way.tags.contains_key("highway") {
                    if way.tags.contains_key("maxspeed") {
                        let _weight = parse_maxspeed(way.tags.get("maxspeed").unwrap().to_string());
                    }
                    // println!("{:?}", way);
                }
            }
        }
    }
    // get all node IDs from ways without duplication
    // store all geo-information about the nodes (also save min and max of long and lat)
    // calculate the time of all ways

    /*
    result of this program:
        int[] source, target, weight
        int[] offset_table
        double[] latitude, longitude
        double max_latitude, min_latitude, max_longitude, min_longitude
    */
}
