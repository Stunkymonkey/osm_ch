use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Point {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Property {
    pub weight: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GeometryRequest {
    // escaping the rust-type command to normal type string
    pub r#type: String,
    pub coordinates: Vec<f32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureRequest {
    // escaping the rust-type command to normal type string
    pub r#type: String,
    pub properties: Option<Property>,
    pub geometry: GeometryRequest,
}

#[derive(Deserialize, Serialize)]
pub struct GeoJsonRequest {
    // escaping the rust-type command to normal type string
    pub r#type: String,
    pub features: Vec<FeatureRequest>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GeometryResponse {
    // escaping the rust-type command to normal type string
    pub r#type: String,
    pub coordinates: Vec<(f32, f32)>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureResponse {
    // escaping the rust-type command to normal type string
    pub r#type: String,
    pub properties: Option<Property>,
    pub geometry: GeometryResponse,
}

#[derive(Deserialize, Serialize)]
pub struct GeoJsonRespone {
    // escaping the rust-type command to normal type string
    pub r#type: String,
    pub features: Vec<FeatureResponse>,
}
