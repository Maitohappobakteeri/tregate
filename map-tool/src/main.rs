mod data;
mod geojson;
mod geometry;
mod map;
mod ui;

use std::fs;

use data::download_collection;
use data::read_height_data_from_file;
use data::write_building_models;
use data::write_building_normals;
use data::write_surface_model;
use data::write_surface_normals;
use geometry::bbox;
use geometry::point;
use map::*;

use crate::data::{read_geojson_from_file, write_height_map};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let resp = read_geojson_from_file("./data/heightdata.json")?;
    // let resp_depth = read_geojson_from_file("./data/depthdata.json")?;
    // let mut resp_building = download_collection("rakennus", "23.75,61.47,23.8,61.52").await?;
    let mut resp_building = vec![
        read_geojson_from_file("./data/rakennus-1.json")?,
        read_geojson_from_file("./data/rakennus-2.json")?,
        read_geojson_from_file("./data/rakennus-3.json")?,
        read_geojson_from_file("./data/rakennus-4.json")?,
        read_geojson_from_file("./data/rakennus-5.json")?,
    ];

    // let mut resp_road = download_collection("tieviiva", "23.75,61.47,23.8,61.52").await?;
    let mut resp_road = vec![
        read_geojson_from_file("./data/tieviiva-1.json")?,
        read_geojson_from_file("./data/tieviiva-2.json")?,
        read_geojson_from_file("./data/tieviiva-3.json")?,
    ];
    // let resp_road = read_geojson_from_file("./data/tiedata.json")?;
    // let mut resp_lake = download_collection("jarvi", "23.7,61.4,23.9,61.6").await?;
    let mut resp_lake = vec![read_geojson_from_file("./data/jarvi-1.json")?];
    let height_data = read_height_data_from_file("./data/heightgrid.txt")?;
    // let respBuild = read_geojson_from_file("./buildingdata.json")?;

    // println!("{:#?}", bbox);

    let smooth_rows = Map::new(height_data, resp_road, resp_lake, resp_building);
    // let smooth_rows = smooth_rows.smooth_map();
    // let smooth_rows = smooth_map(smooth_rows);
    // let smooth_rows = smooth_map(smooth_rows);
    // let smooth_rows = blur_dark_spots(smooth_rows);
    fs::create_dir_all("output/")?;
    write_height_map("output/map.json", &smooth_rows)?;
    write_surface_model("output/height_model.json", &smooth_rows, 2)?;
    write_surface_normals("output/height_normals.json", &smooth_rows, 2)?;
    write_building_models("output/building_models.json", &smooth_rows)?;
    write_building_normals("output/building_normals.json", &smooth_rows)?;
    Ok(())
}
