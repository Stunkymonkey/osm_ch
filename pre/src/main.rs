extern crate osmpbfreader;

use osmpbfreader::{groups, primitive_block_from_blob};

fn main() {
    let filename = std::env::args_os().nth(1).unwrap();
    let path = std::path::Path::new(&filename);
    let r = std::fs::File::open(&path).unwrap();
    let mut pbf = osmpbfreader::OsmPbfReader::new(r);
    // first store all way-IDs having the "highway" tag in binary heap?
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        for group in block.get_primitivegroup().iter() {
            for way in groups::ways(&group, &block) {
                if way.tags.contains_key("highway") {
                    println!("{:?}", way);
                }
            }
        }
    }
    // get all node IDs without duplication from ways
    // store all geo-information about the nodes
}
