pub fn dijkstra(start: u32, end: u32) -> Vec<u32> {
    return Vec::<u32>::new();
}

pub fn get_point_id(lat: f32, long: f32) -> u32 {
    // calculate distance to all points and get minimum
    return 0;
}

pub fn get_coordinates(id: u32) -> (f32, f32) {
    return (0.0, 0.0);
}

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
