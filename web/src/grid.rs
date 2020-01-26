use super::*;

pub fn get_closest_point(lat: f32, lng: f32, nodes: &Vec<Node>) -> usize {
    //TODO
    return 0;
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

/// converts node ids to node-coordinates
pub fn get_coordinates(path: Vec<NodeId>, nodes: &Vec<Node>) -> Vec<Node> {
    return path.par_iter().map(|x| nodes[*x]).collect::<Vec<Node>>();
}
