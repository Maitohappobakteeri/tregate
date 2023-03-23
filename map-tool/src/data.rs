use std::fs::write;
use std::io::{BufRead, BufWriter, Write};
use std::vec;
use std::{collections::HashMap, error::Error, fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};

use crate::geojson::GeoJSON;
use crate::geometry::point::Vec3;
use crate::map::Map;
use crate::ui;

pub fn read_json_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn read_height_data_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
    println!("Parsing height data");
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    rows.push(Vec::new());
    for line in reader
        .split(b' ')
        .map(|x| String::from_utf8(x.unwrap()).unwrap_or(String::new()))
        .flat_map(|x| x.split("\r\n").map(|y| String::from(y)).collect::<Vec<_>>())
        .filter(|x| x.len() > 0)
        .skip(12)
    {
        if (rows.last().unwrap().len() >= 1468) {
            rows.push(Vec::new());
            ui::print_progress_bar(rows.len() as f64 / 2716.0);
        }
        let row = rows.last_mut().unwrap();
        row.push(line.parse()?);
    }
    ui::print_progress_bar_completed();
    Ok(rows)
}

pub fn read_geojson_from_file<P: AsRef<Path>>(path: P) -> Result<GeoJSON, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn write_height_map<P: AsRef<Path>>(path: P, map: &Map) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string(&(map.height_map(), map.class_map()))?;
    write(path, json).expect("Unable to write file");
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct ModelOutput {
    vertices: Vec<Vec<f64>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BuildingModelOutput {
    vertices: Vec<Vec<Vec<f64>>>,
}

pub fn write_surface_model<P: AsRef<Path>>(
    path: P,
    map: &Map,
    scale: usize,
) -> Result<(), Box<dyn Error>> {
    let f = File::create(path).expect("Unable to create file");
    let f = BufWriter::new(f);

    let mut vertices: Vec<Vec<f64>> = Vec::new();
    let height_map = map.height_map();
    for (y, row) in height_map[..height_map.len() - 1].iter().enumerate() {
        if y % scale != 0 || y + scale >= height_map[..height_map.len() - 1].len() {
            continue;
        }

        for (x, _) in row[..row.len() - 1].iter().enumerate() {
            if x % scale != 0 || x + scale >= row[..row.len() - 1].len() {
                continue;
            }

            let rectSize = scale as f64;
            vertices.push(vec![x as f64, y as f64, height_map[y][x] as f64, 1.0]);
            vertices.push(vec![
                x as f64 + rectSize,
                y as f64,
                height_map[y][x + scale] as f64,
                1.0,
            ]);
            vertices.push(vec![
                x as f64,
                y as f64 + rectSize,
                height_map[y + scale][x] as f64,
                1.0,
            ]);
            vertices.push(vec![
                x as f64,
                y as f64 + rectSize,
                height_map[y + scale][x] as f64,
                1.0,
            ]);
            vertices.push(vec![
                x as f64 + rectSize,
                y as f64 + rectSize,
                height_map[y + scale][x + scale] as f64,
                1.0,
            ]);
            vertices.push(vec![
                x as f64 + rectSize,
                y as f64,
                height_map[y][x + scale] as f64,
                1.0,
            ]);
        }
    }
    let output = ModelOutput { vertices };
    serde_json::to_writer(f, &output)?;
    Ok(())
}

pub fn write_building_models<P: AsRef<Path>>(path: P, map: &Map) -> Result<(), Box<dyn Error>> {
    let f = File::create(path).expect("Unable to create file");
    let f = BufWriter::new(f);

    let mut buildingVertices: Vec<Vec<Vec<f64>>> = Vec::new();
    let buildings = &map.buildings;
    for building in buildings.iter() {
        let mut vertices: Vec<Vec<f64>> = Vec::new();
        let bbox = building.bbox();
        let a = map.to_tile_coords(bbox.a.x, bbox.a.y);
        let b = map.to_tile_coords(bbox.b.x, bbox.b.y);

        // -1.0f,-1.0f,-1.0f, // triangle 1 : begin
        // -1.0f,-1.0f, 1.0f,
        // -1.0f, 1.0f, 1.0f, // triangle 1 : end
        const LOWER: f64 = 1.4;
        const UPPER: f64 = 1.5;
        vertices.push(vec![a.0, a.1, LOWER, 1.0]);
        vertices.push(vec![a.0, a.1, UPPER, 1.0]);
        vertices.push(vec![a.0, b.1, UPPER, 1.0]);

        // 1.0f, 1.0f,-1.0f, // triangle 2 : begin
        // -1.0f,-1.0f,-1.0f,
        // -1.0f, 1.0f,-1.0f, // triangle 2 : end
        vertices.push(vec![b.0, b.1, LOWER, 1.0]);
        vertices.push(vec![a.0, a.1, LOWER, 1.0]);
        vertices.push(vec![a.0, b.1, LOWER, 1.0]);

        // 1.0f,-1.0f, 1.0f,
        // -1.0f,-1.0f,-1.0f,
        // 1.0f,-1.0f,-1.0f,
        vertices.push(vec![b.0, a.1, UPPER, 1.0]);
        vertices.push(vec![a.0, a.1, LOWER, 1.0]);
        vertices.push(vec![b.0, a.1, LOWER, 1.0]);

        // 1.0f, 1.0f,-1.0f,
        // 1.0f,-1.0f,-1.0f,
        // -1.0f,-1.0f,-1.0f,
        vertices.push(vec![b.0, b.1, LOWER, 1.0]);
        vertices.push(vec![b.0, a.1, LOWER, 1.0]);
        vertices.push(vec![a.0, a.1, LOWER, 1.0]);

        // -1.0f,-1.0f,-1.0f,
        // -1.0f, 1.0f, 1.0f,
        // -1.0f, 1.0f,-1.0f,
        vertices.push(vec![a.0, a.1, LOWER, 1.0]);
        vertices.push(vec![a.0, b.1, UPPER, 1.0]);
        vertices.push(vec![a.0, b.1, LOWER, 1.0]);

        // 1.0f,-1.0f, 1.0f,
        // -1.0f,-1.0f, 1.0f,
        // -1.0f,-1.0f,-1.0f,
        vertices.push(vec![b.0, a.1, UPPER, 1.0]);
        vertices.push(vec![a.0, a.1, UPPER, 1.0]);
        vertices.push(vec![a.0, a.1, LOWER, 1.0]);

        // -1.0f, 1.0f, 1.0f,
        // -1.0f,-1.0f, 1.0f,
        // 1.0f,-1.0f, 1.0f,
        vertices.push(vec![a.0, b.1, UPPER, 1.0]);
        vertices.push(vec![a.0, a.1, UPPER, 1.0]);
        vertices.push(vec![b.0, b.1, UPPER, 1.0]);

        // 1.0f, 1.0f, 1.0f,
        // 1.0f,-1.0f,-1.0f,
        // 1.0f, 1.0f,-1.0f,
        vertices.push(vec![b.0, b.1, UPPER, 1.0]);
        vertices.push(vec![b.0, a.1, LOWER, 1.0]);
        vertices.push(vec![b.0, b.1, LOWER, 1.0]);

        // 1.0f,-1.0f,-1.0f,
        // 1.0f, 1.0f, 1.0f,
        // 1.0f,-1.0f, 1.0f,
        vertices.push(vec![b.0, a.1, LOWER, 1.0]);
        vertices.push(vec![b.0, b.1, UPPER, 1.0]);
        vertices.push(vec![b.0, a.1, UPPER, 1.0]);

        // 1.0f, 1.0f, 1.0f,
        // 1.0f, 1.0f,-1.0f,
        // -1.0f, 1.0f,-1.0f,
        vertices.push(vec![b.0, b.1, UPPER, 1.0]);
        vertices.push(vec![b.0, b.1, LOWER, 1.0]);
        vertices.push(vec![a.0, b.1, LOWER, 1.0]);

        // 1.0f, 1.0f, 1.0f,
        // -1.0f, 1.0f,-1.0f,
        // -1.0f, 1.0f, 1.0f,
        vertices.push(vec![b.0, b.1, UPPER, 1.0]);
        vertices.push(vec![a.0, b.1, LOWER, 1.0]);
        vertices.push(vec![a.0, b.1, UPPER, 1.0]);

        // 1.0f, 1.0f, 1.0f,
        // -1.0f, 1.0f, 1.0f,
        // 1.0f,-1.0f, 1.0f
        vertices.push(vec![b.0, b.1, UPPER, 1.0]);
        vertices.push(vec![a.0, b.1, UPPER, 1.0]);
        vertices.push(vec![b.0, a.1, UPPER, 1.0]);

        buildingVertices.push(vertices);
    }
    let output = BuildingModelOutput {
        vertices: buildingVertices,
    };
    serde_json::to_writer(f, &output)?;
    Ok(())
}

pub fn write_building_normals<P: AsRef<Path>>(path: P, map: &Map) -> Result<(), Box<dyn Error>> {
    let f = File::create(path).expect("Unable to create file");
    let f = BufWriter::new(f);

    let mut buildingVertices: Vec<Vec<Vec<f64>>> = Vec::new();
    let buildings = &map.buildings;
    for building in buildings.iter() {
        let mut vertices: Vec<Vec<f64>> = Vec::new();
        let bbox = building.bbox();
        let a = map.to_tile_coords(bbox.a.x, bbox.a.y);
        let b = map.to_tile_coords(bbox.b.x, bbox.b.y);

        // -1.0f,-1.0f,-1.0f, // triangle 1 : begin
        // -1.0f,-1.0f, 1.0f,
        // -1.0f, 1.0f, 1.0f, // triangle 1 : end
        vertices.push(vec![-1.0, 0.0, 0.0]);
        vertices.push(vec![-1.0, 0.0, 0.0]);
        vertices.push(vec![-1.0, 0.0, 0.0]);

        // 1.0f, 1.0f,-1.0f, // triangle 2 : begin
        // -1.0f,-1.0f,-1.0f,
        // -1.0f, 1.0f,-1.0f, // triangle 2 : end
        vertices.push(vec![0.0, 0.0, -1.0]);
        vertices.push(vec![0.0, 0.0, -1.0]);
        vertices.push(vec![0.0, 0.0, -1.0]);

        // 1.0f,-1.0f, 1.0f,
        // -1.0f,-1.0f,-1.0f,
        // 1.0f,-1.0f,-1.0f,
        vertices.push(vec![0.0, -1.0, 0.0]);
        vertices.push(vec![0.0, -1.0, 0.0]);
        vertices.push(vec![0.0, -1.0, 0.0]);

        // 1.0f, 1.0f,-1.0f,
        // 1.0f,-1.0f,-1.0f,
        // -1.0f,-1.0f,-1.0f,
        vertices.push(vec![0.0, 0.0, -1.0]);
        vertices.push(vec![0.0, 0.0, -1.0]);
        vertices.push(vec![0.0, 0.0, -1.0]);

        // -1.0f,-1.0f,-1.0f,
        // -1.0f, 1.0f, 1.0f,
        // -1.0f, 1.0f,-1.0f,
        vertices.push(vec![-1.0, 0.0, 0.0]);
        vertices.push(vec![-1.0, 0.0, 0.0]);
        vertices.push(vec![-1.0, 0.0, 0.0]);

        // 1.0f,-1.0f, 1.0f,
        // -1.0f,-1.0f, 1.0f,
        // -1.0f,-1.0f,-1.0f,
        vertices.push(vec![0.0, -1.0, 0.0]);
        vertices.push(vec![0.0, -1.0, 0.0]);
        vertices.push(vec![0.0, -1.0, 0.0]);

        // -1.0f, 1.0f, 1.0f,
        // -1.0f,-1.0f, 1.0f,
        // 1.0f,-1.0f, 1.0f,
        vertices.push(vec![0.0, 0.0, 1.0]);
        vertices.push(vec![0.0, 0.0, 1.0]);
        vertices.push(vec![0.0, 0.0, 1.0]);

        // 1.0f, 1.0f, 1.0f,
        // 1.0f,-1.0f,-1.0f,
        // 1.0f, 1.0f,-1.0f,
        vertices.push(vec![1.0, 0.0, 0.0]);
        vertices.push(vec![1.0, 0.0, 0.0]);
        vertices.push(vec![1.0, 0.0, 0.0]);

        // 1.0f,-1.0f,-1.0f,
        // 1.0f, 1.0f, 1.0f,
        // 1.0f,-1.0f, 1.0f,
        vertices.push(vec![1.0, 0.0, 0.0]);
        vertices.push(vec![1.0, 0.0, 0.0]);
        vertices.push(vec![1.0, 0.0, 0.0]);

        // 1.0f, 1.0f, 1.0f,
        // 1.0f, 1.0f,-1.0f,
        // -1.0f, 1.0f,-1.0f,
        vertices.push(vec![0.0, 1.0, 0.0]);
        vertices.push(vec![0.0, 1.0, 0.0]);
        vertices.push(vec![0.0, 1.0, 0.0]);

        // 1.0f, 1.0f, 1.0f,
        // -1.0f, 1.0f,-1.0f,
        // -1.0f, 1.0f, 1.0f,
        vertices.push(vec![0.0, 1.0, 0.0]);
        vertices.push(vec![0.0, 1.0, 0.0]);
        vertices.push(vec![0.0, 1.0, 0.0]);

        // 1.0f, 1.0f, 1.0f,
        // -1.0f, 1.0f, 1.0f,
        // 1.0f,-1.0f, 1.0f
        vertices.push(vec![0.0, 0.0, -1.0]);
        vertices.push(vec![0.0, 0.0, -1.0]);
        vertices.push(vec![0.0, 0.0, -1.0]);

        buildingVertices.push(vertices);
    }
    let output = BuildingModelOutput {
        vertices: buildingVertices,
    };
    serde_json::to_writer(f, &output)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct NormalOutput {
    normals: Vec<Vec<f64>>,
}

fn surface_normal_from_vectors(origin: &Vec3, a: &Vec3, b: &Vec3) -> Vec3 {
    let mut n = (a - origin).cross(&(b - origin)).normalize();
    if n.z < 0.0 {
        n = &n * -1.0;
    }
    n
}

fn get_height_or_average(height_map: &Vec<Vec<f64>>, x: i64, y: i64, average: f64) -> f64 {
    if y >= height_map.len() as i64 || y < 0 {
        return average;
    }
    let row = &height_map[y as usize];
    if x >= row.len() as i64 || x < 0 {
        return average;
    }
    return height_map[y as usize][x as usize];
}

fn normal_for_point(
    height_map: &Vec<Vec<f64>>,
    distance_diff: f64,
    scale: usize,
    x: usize,
    y: usize,
    average: f64,
) -> Vec3 {
    let x = x as i64;
    let y = y as i64;
    let scale = scale as i64;
    let normals = vec![
        surface_normal_from_vectors(
            &Vec3 {
                x: x as f64,
                y: y as f64,
                z: get_height_or_average(height_map, x, y, average),
            },
            &Vec3 {
                x: x as f64 - distance_diff,
                y: y as f64,
                z: get_height_or_average(height_map, x - scale, y, average),
            },
            &Vec3 {
                x: x as f64 - distance_diff,
                y: y as f64 - distance_diff,
                z: get_height_or_average(height_map, x - scale, y - scale, average),
            },
        ),
        surface_normal_from_vectors(
            &Vec3 {
                x: x as f64,
                y: y as f64,
                z: get_height_or_average(height_map, x, y, average),
            },
            &Vec3 {
                x: x as f64 - distance_diff,
                y: y as f64 - distance_diff,
                z: get_height_or_average(height_map, x - scale, y - scale, average),
            },
            &Vec3 {
                x: x as f64,
                y: y as f64 - distance_diff,
                z: get_height_or_average(height_map, x, y - scale, average),
            },
        ),
        //
        surface_normal_from_vectors(
            &Vec3 {
                x: x as f64,
                y: y as f64,
                z: get_height_or_average(height_map, x, y, average),
            },
            &Vec3 {
                x: x as f64,
                y: y as f64 - distance_diff,
                z: get_height_or_average(height_map, x, y - scale, average),
            },
            &Vec3 {
                x: x as f64 + distance_diff,
                y: y as f64 - distance_diff,
                z: get_height_or_average(height_map, x + scale, y - scale, average),
            },
        ),
        surface_normal_from_vectors(
            &Vec3 {
                x: x as f64,
                y: y as f64,
                z: get_height_or_average(height_map, x, y, average),
            },
            &Vec3 {
                x: x as f64 + distance_diff,
                y: y as f64 - distance_diff,
                z: get_height_or_average(height_map, x + scale, y - scale, average),
            },
            &Vec3 {
                x: x as f64 + distance_diff,
                y: y as f64,
                z: get_height_or_average(height_map, x + scale, y, average),
            },
        ),
        //
        surface_normal_from_vectors(
            &Vec3 {
                x: x as f64,
                y: y as f64,
                z: get_height_or_average(height_map, x, y, average),
            },
            &Vec3 {
                x: x as f64 + distance_diff,
                y: y as f64,
                z: get_height_or_average(height_map, x + scale, y, average),
            },
            &Vec3 {
                x: x as f64 + distance_diff,
                y: y as f64 + distance_diff,
                z: get_height_or_average(height_map, x + scale, y + scale, average),
            },
        ),
        surface_normal_from_vectors(
            &Vec3 {
                x: x as f64,
                y: y as f64,
                z: get_height_or_average(height_map, x, y, average),
            },
            &Vec3 {
                x: x as f64 + distance_diff,
                y: y as f64 + distance_diff,
                z: get_height_or_average(height_map, x + scale, y + scale, average),
            },
            &Vec3 {
                x: x as f64,
                y: y as f64 + distance_diff,
                z: get_height_or_average(height_map, x, y + scale, average),
            },
        ),
        //
        surface_normal_from_vectors(
            &Vec3 {
                x: x as f64,
                y: y as f64,
                z: get_height_or_average(height_map, x, y, average),
            },
            &Vec3 {
                x: x as f64,
                y: y as f64 + distance_diff,
                z: get_height_or_average(height_map, x, y + scale, average),
            },
            &Vec3 {
                x: x as f64 - distance_diff,
                y: y as f64 + distance_diff,
                z: get_height_or_average(height_map, x - scale, y + scale, average),
            },
        ),
        surface_normal_from_vectors(
            &Vec3 {
                x: x as f64,
                y: y as f64,
                z: get_height_or_average(height_map, x, y, average),
            },
            &Vec3 {
                x: x as f64 - distance_diff,
                y: y as f64 + distance_diff,
                z: get_height_or_average(height_map, x - scale, y + scale, average),
            },
            &Vec3 {
                x: x as f64 - distance_diff,
                y: y as f64,
                z: get_height_or_average(height_map, x - scale, y, average),
            },
        ),
    ];

    let mut sum = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    for n in normals.iter() {
        sum = &sum + n;
    }
    sum = &sum * (1.0 / normals.len() as f64);
    sum
}

pub fn write_surface_normals<P: AsRef<Path>>(
    path: P,
    map: &Map,
    scale: usize,
) -> Result<(), Box<dyn Error>> {
    let f = File::create(path).expect("Unable to create file");
    let f = BufWriter::new(f);

    let mut normals: Vec<Vec<f64>> = Vec::new();
    let height_map = map.height_map();
    for (y, row) in height_map[..height_map.len() - 1].iter().enumerate() {
        if y % scale != 0 || y + scale >= height_map[..height_map.len() - 1].len() {
            continue;
        }

        for (x, _) in row[..row.len() - 1].iter().enumerate() {
            if x % scale != 0 || x + scale >= row[..row.len() - 1].len() {
                continue;
            }

            let average = 1.6;
            let distance_diff = 0.1;
            // let mut normal = (Vec3 {
            //     x: x as f64,
            //     y: y as f64,
            //     z: height_map[y][x] as f64,
            // })
            // .cross(&Vec3 {
            //     x: x as f64,
            //     y: y as f64 + distanceDiff,
            //     z: height_map[y + scale][x] as f64,
            // })
            // .normalize();

            // if normal.z < 0.0 {
            //     normal = &normal * -1.0;
            // }

            // // let normal = Vec3 {
            // //     x: 0.0,
            // //     y: 0.0,
            // //     z: 1.0,
            // // };
            // normals.push(vec![normal.x, normal.y, normal.z]);
            // normals.push(vec![normal.x, normal.y, normal.z]);
            // normals.push(vec![normal.x, normal.y, normal.z]);

            // let mut normal = (Vec3 {
            //     x: x as f64 + distanceDiff,
            //     y: y as f64 + distanceDiff,
            //     z: height_map[y + scale][x + scale] as f64,
            // })
            // .cross(&Vec3 {
            //     x: x as f64 + distanceDiff,
            //     y: y as f64,
            //     z: height_map[y][x + scale] as f64,
            // })
            // .normalize();

            // if normal.z < 0.0 {
            //     normal = &normal * -1.0;
            // }

            // // let normal = Vec3 {
            // //     x: 0.0,
            // //     y: 0.0,
            // //     z: 1.0,
            // // };
            // normals.push(vec![normal.x, normal.y, normal.z]);
            // normals.push(vec![normal.x, normal.y, normal.z]);
            // normals.push(vec![normal.x, normal.y, normal.z]);

            let normal = normal_for_point(&height_map, distance_diff, scale, x, y, average);
            normals.push(vec![normal.x, normal.y, normal.z]);
            let normal = normal_for_point(&height_map, distance_diff, scale, x + scale, y, average);
            normals.push(vec![normal.x, normal.y, normal.z]);
            let normal = normal_for_point(&height_map, distance_diff, scale, x, y + scale, average);
            normals.push(vec![normal.x, normal.y, normal.z]);

            let normal = normal_for_point(&height_map, distance_diff, scale, x, y + scale, average);
            normals.push(vec![normal.x, normal.y, normal.z]);
            let normal = normal_for_point(
                &height_map,
                distance_diff,
                scale,
                x + scale,
                y + scale,
                average,
            );
            normals.push(vec![normal.x, normal.y, normal.z]);
            let normal = normal_for_point(&height_map, distance_diff, scale, x + scale, y, average);
            normals.push(vec![normal.x, normal.y, normal.z]);
        }
    }
    let output = NormalOutput { normals };
    serde_json::to_writer(f, &output)?;
    Ok(())
}

pub async fn download_collection(
    collection_name: &str,
    bbox: &str,
) -> Result<Vec<GeoJSON>, Box<dyn Error>> {
    let config = read_json_from_file("../.local_config")?;
    let api_key = config.get("apiKey").ok_or("Missing API key from config")?;
    let client = reqwest::Client::new();
    let mut responses = Vec::new();

    let mut pages = 0;
    let mut href = Some(format!("https://avoin-paikkatieto.maanmittauslaitos.fi/maastotiedot/features/v1/collections/{collection_name}/items?bbox={bbox}"));
    while href.is_some() {
        pages += 1;
        let response = client
            .get(href.unwrap().clone())
            .basic_auth(api_key, Some(""))
            .send()
            .await?
            .json::<GeoJSON>()
            // .json::<serde_json::Value>()
            .await?;
        let nextLink = response
            .links
            .as_ref()
            .map(|x| x.iter().filter(|l| l.rel == "next").next())
            .flatten();
        href = nextLink
            .map(|l| {
                if pages < 10 {
                    Some(l.href.clone())
                } else {
                    None
                }
            })
            .flatten();

        let json = serde_json::to_string(&response)?;
        write(format!("./data/{collection_name}-{pages}.json"), json)
            .expect("Unable to write file");

        responses.push(response);
    }

    // korkeuskayra
    // syvyyskayra
    // rakennus
    Ok(responses)
}
