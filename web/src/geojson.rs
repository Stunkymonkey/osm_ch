use serde::{Deserialize, Serialize};

// r#type for escaping the rust-type command to normal type string

#[derive(Deserialize, Serialize)]
pub struct Point {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Property {
    pub weight: String,
}

// request are two points
#[derive(Deserialize, Serialize, Debug)]
pub struct GeometryRequest {
    pub r#type: String,
    pub coordinates: Vec<f32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureRequest {
    pub r#type: String,
    pub properties: Option<Property>,
    pub geometry: GeometryRequest,
}

#[derive(Deserialize, Serialize)]
pub struct GeoJsonRequest {
    pub r#type: String,
    pub features: Vec<FeatureRequest>,
}

// response is array of tuples
#[derive(Deserialize, Serialize, Debug)]
pub struct GeometryResponse {
    pub r#type: String,
    pub coordinates: Vec<(f32, f32)>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureResponse {
    pub r#type: String,
    pub properties: Option<Property>,
    pub geometry: GeometryResponse,
}

#[derive(Deserialize, Serialize)]
pub struct GeoJsonRespone {
    pub r#type: String,
    pub features: Vec<FeatureResponse>,
}
