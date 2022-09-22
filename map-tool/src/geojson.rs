use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Coordinates {
    points(Vec<(f64, f64)>),
    polygon(Vec<Vec<(f64, f64)>>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoJSONGeometry {
    pub coordinates: Coordinates,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoJSONProperties {
    pub korkeusarvo: Option<i64>,
    pub syvyysarvo: Option<i64>,
    pub pohjankorkeus: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoJSONFeatures {
    pub geometry: GeoJSONGeometry,
    pub properties: GeoJSONProperties,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoJSONLinks {
    pub href: String,
    pub rel: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoJSON {
    pub features: Vec<GeoJSONFeatures>,
    pub links: Option<Vec<GeoJSONLinks>>,
}
