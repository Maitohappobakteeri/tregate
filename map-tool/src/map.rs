use std::io::stdout;
use std::io::Write;

use crate::bbox;
use crate::bbox::BboxTree;
use crate::bbox::HasCoords;
use crate::geojson;
use crate::geojson::Coordinates;
use crate::geojson::GeoJSON;
use crate::geometry::point::LineSegment;
use crate::geometry::point::Polygon;
use crate::point::Point;
use crate::ui;

const MAP_SIZE: i64 = 512;
const ZOOM: f64 = 0.5;
const LAT_MAX: f64 = 61.515 - (0.05 - MAP_SIZE_D) * 0.5;
const LONG_MIN: f64 = 23.745 + (0.05 - MAP_SIZE_D) * 0.5;
const MAP_SIZE_D: f64 = 0.05 * ZOOM;

#[derive(Debug)]
pub struct HeightPoint {
    pub height: i64,
    pub coords: Point,
}

impl HasCoords for HeightPoint {
    fn fits_into(self: &HeightPoint, bbox: &bbox::Bbox) -> bool {
        bbox.contains(&self.coords)
    }
}

impl Clone for HeightPoint {
    fn clone(self: &HeightPoint) -> HeightPoint {
        HeightPoint {
            height: self.height,
            coords: Point {
                x: self.coords.x,
                y: self.coords.y,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassPoint {
    pub class: MapTileClass,
    pub coords: Point,
}

impl HasCoords for ClassPoint {
    fn fits_into(&self, bbox: &bbox::Bbox) -> bool {
        bbox.contains(&self.coords)
    }
}

#[derive(Clone)]
enum Shape {
    LINE_SEGMENT(LineSegment, MapTileClass),
    POLYGON(Polygon, MapTileClass),
}

impl HasCoords for Shape {
    fn fits_into(&self, bbox: &bbox::Bbox) -> bool {
        let b = match self {
            Shape::LINE_SEGMENT(x, ..) => x.bbox(),
            Shape::POLYGON(x, ..) => x.bbox().clone(),
        };

        bbox.overlaps(&b)
    }
}

impl BboxTree<HeightPoint> {
    pub fn max_height(self: &BboxTree<HeightPoint>) -> i64 {
        match self {
            BboxTree::Interior { ref children, .. } => *[
                (*children.0).max_height(),
                (*children.1).max_height(),
                (*children.2).max_height(),
                (*children.3).max_height(),
            ]
            .iter()
            .max()
            .unwrap_or(&0),
            BboxTree::Leaf { ref items, .. } => items.iter().map(|h| h.height).max().unwrap_or(0),
        }
    }

    pub fn average_height(self: &BboxTree<HeightPoint>) -> i64 {
        match self {
            BboxTree::Interior { ref children, .. } => {
                [
                    (*children.0).average_height(),
                    (*children.1).average_height(),
                    (*children.2).average_height(),
                    (*children.3).average_height(),
                ]
                .iter()
                .filter(|x| **x > 0)
                .sum::<i64>()
                    / 4
            }
            BboxTree::Leaf { ref items, .. } => {
                if items.len() == 0 {
                    return 0;
                }

                items.iter().map(|h| h.height).sum::<i64>() / items.len() as i64
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MapTileClass {
    WATER,
    BUILDING,
    ROAD,
    EMPTY,
}

pub struct MapTile {
    pub height: i64,
    pub class: MapTileClass,
}

pub struct Map {
    pub tiles: Vec<Vec<MapTile>>,
    pub buildings: Vec<Polygon>,
}

impl Map {
    pub fn new(
        heightData: Vec<Vec<f64>>,
        roads: Vec<GeoJSON>,
        water: Vec<GeoJSON>,
        buildings: Vec<GeoJSON>,
    ) -> Map {
        let mut bbox: BboxTree<HeightPoint> = BboxTree::Leaf {
            bbox: bbox::Bbox {
                // a: bbox::Point { x: 23.37, y: 61.19 },
                // b: bbox::Point { x: 23.47, y: 61.39 }
                a: Point { x: 23.0, y: 60.0 },
                b: Point { x: 24.0, y: 62.0 },
            },
            items: Vec::new(),
        };

        let mut bboxClass: BboxTree<ClassPoint> = BboxTree::Leaf {
            bbox: bbox::Bbox {
                // a: bbox::Point { x: 23.37, y: 61.19 },
                // b: bbox::Point { x: 23.47, y: 61.39 }
                a: Point { x: 23.0, y: 60.0 },
                b: Point { x: 24.0, y: 62.0 },
            },
            items: Vec::new(),
        };
        let mut stdout = stdout();

        let roadData = GeoJSON {
            features: roads.into_iter().flat_map(|d| d.features).collect(),
            links: None,
        };

        println!("Adding roads");
        let mut roadSegments = Vec::new();
        let mut segment_counter = 0;
        let points = roadData
            .features
            .iter()
            .map(|f| {
                (
                    f,
                    match (&f.geometry.coordinates) {
                        geojson::Coordinates::points(x) => x.clone(),
                        geojson::Coordinates::polygon(x) => {
                            // separate to features?!
                            x.iter().flat_map(|x| x.clone()).collect::<Vec<_>>()
                        }
                    },
                )
            })
            .map(|(f, coords)| {
                coords
                    .iter()
                    .map(|c| HeightPoint {
                        height: f
                            .properties
                            .korkeusarvo
                            .or(f.properties.syvyysarvo.map(|x| x * 100))
                            .or(f.properties.pohjankorkeus)
                            .unwrap_or(10_000),
                        coords: Point { x: c.0, y: c.1 },
                    })
                    .collect::<Vec<_>>()
            });
        for line_segment in points {
            segment_counter += 1;
            for ii in 0..(line_segment.len() - 1) {
                let point_a = &line_segment[ii];
                let point_b = &line_segment[ii + 1];

                roadSegments.push(LineSegment {
                    a: Point {
                        x: point_a.coords.x,
                        y: point_a.coords.y,
                    },
                    b: Point {
                        x: point_b.coords.x,
                        y: point_b.coords.y,
                    },
                });

                if ii % 5 == 0 {
                    let progress = segment_counter as f64 / roadData.features.len() as f64
                        + (ii as f64 / (line_segment.len() as f64).powf(2.0));
                    ui::print_progress_bar(progress);
                    stdout.flush().unwrap();
                }
            }
        }
        ui::print_progress_bar_completed();

        // println!("Adding lake outline");
        // let mut segment_counter = 0;
        // let points = water
        //     .iter()
        //     .flat_map(|d| &d.features)
        //     .map(|f| {
        //         (
        //             f,
        //             match &f.geometry.coordinates {
        //                 geojson::Coordinates::points(x) => x.clone(),
        //                 geojson::Coordinates::polygon(x) => {
        //                     // separate to features?!
        //                     x.iter().flat_map(|x| x.clone()).collect::<Vec<_>>()
        //                 }
        //             },
        //         )
        //     })
        //     .map(|(f, coords)| {
        //         coords
        //             .iter()
        //             .map(|c| HeightPoint {
        //                 height: f
        //                     .properties
        //                     .korkeusarvo
        //                     .or(f.properties.syvyysarvo.map(|x| x * 100))
        //                     .or(f.properties.pohjankorkeus)
        //                     .unwrap_or(10_000),
        //                 coords: Point { x: c.0, y: c.1 },
        //             })
        //             .collect::<Vec<_>>()
        //     });
        // for line_segment in points {
        //     segment_counter += 1;
        //     for ii in 1..line_segment.len() {
        //         let point_a = &line_segment[ii - 1];
        //         let point_b = &line_segment[ii];

        //         let d_x = point_b.coords.x - point_a.coords.x;
        //         let d_y = point_b.coords.y - point_a.coords.y;
        //         let distance = (d_y.powf(2.0) + d_x.powf(2.0)).sqrt();

        //         let mut travelled = 0.0;
        //         while travelled < distance {
        //             bboxClass = bboxClass.add_item(
        //                 &ClassPoint {
        //                     class: MapTileClass::WATER,
        //                     coords: Point {
        //                         x: point_a.coords.x + travelled * d_x,
        //                         y: point_a.coords.y + travelled * d_y,
        //                     },
        //                 },
        //                 0,
        //             );
        //             travelled += 0.0001;
        //         }

        //         if ii % 5 == 0 {
        //             let progress = segment_counter as f64 / roadData.features.len() as f64
        //                 + (ii as f64 / (line_segment.len() as f64).powf(2.0));
        //             ui::print_progress_bar(progress);
        //             stdout.flush().unwrap();
        //         }
        //     }
        // }
        // ui::print_progress_bar_completed();

        println!("Adding lakes");
        let lakePolys = water
            .into_iter()
            .flat_map(|d| d.features)
            .map(|f| {
                if let Coordinates::polygon(points) = f.geometry.coordinates {
                    return Some(
                        points
                            .iter()
                            .map(|p| {
                                Polygon::new(
                                    p.iter()
                                        // .flat_map(|x| x)
                                        .map(|p| Point { x: p.0, y: p.1 })
                                        .collect(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    );
                }
                None
            })
            .filter_map(|x| x)
            .flat_map(|x| x)
            .collect::<Vec<_>>();

        println!("Adding buildings");
        let buildingPolys = buildings
            .into_iter()
            .flat_map(|d| d.features)
            .map(|f| {
                if let Coordinates::polygon(points) = f.geometry.coordinates {
                    return Some(
                        points
                            .iter()
                            .map(|p| {
                                Polygon::new(
                                    p.iter()
                                        // .flat_map(|x| x)
                                        .map(|p| Point { x: p.0, y: p.1 })
                                        .collect(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    );
                }
                None
            })
            .filter_map(|x| x)
            .flat_map(|x| x)
            .collect::<Vec<_>>();

        println!("Adding height data points");
        for x in 0..1468 {
            for y in 0..2716 {
                bbox = bbox.add_item(
                    &HeightPoint {
                        height: (heightData[y][x] * 350.0 / 2.0) as i64,
                        coords: finnish_coords_to_global(
                            6819455.0 + ((y as f64) / 2716.0) * (6824888.0 - 6819455.0),
                            326874.0 + ((x as f64) / 1468.0) * (329810.0 - 326874.0),
                        ),
                    },
                    0,
                );
            }
            ui::print_progress_bar((x as f64) / 1468.0);
        }
        ui::print_progress_bar_completed();

        let heightRows = build_height_map(&bbox);
        let classRows = build_class_map(&bboxClass, &lakePolys, &roadSegments, &buildingPolys);

        let mut tiles = Vec::new();
        for y in 0..(MAP_SIZE as usize) {
            tiles.push(Vec::new());
            for x in 0..(MAP_SIZE as usize) {
                tiles[y].push(MapTile {
                    class: classRows[y][x].clone(),
                    height: heightRows[y][x],
                });
            }
        }

        Map {
            tiles,
            buildings: buildingPolys,
        }
    }

    pub fn height_map(&self) -> Vec<Vec<f64>> {
        self.tiles
            .iter()
            .map(|row| {
                row.iter()
                    .map(|tile| tile.height as f64 / 100_00.0)
                    .collect()
            })
            .collect()
    }

    pub fn class_map(&self) -> Vec<Vec<String>> {
        self.tiles
            .iter()
            .map(|row| {
                row.iter()
                    .map(|tile| match tile.class {
                        MapTileClass::BUILDING => String::from("BUILDING"),
                        MapTileClass::WATER => String::from("WATER"),
                        MapTileClass::ROAD => String::from("ROAD"),
                        MapTileClass::EMPTY => String::from("EMPTY"),
                    })
                    .collect()
            })
            .collect()
    }

    pub fn to_tile_coords(&self, x: f64, y: f64) -> (f64, f64) {
        // let longitude = ((x - LONG_MIN) / MAP_SIZE_D) * MAP_SIZE as f64;
        // let latitude = ((y - LAT_MAX + MAP_SIZE_D) / MAP_SIZE_D) * MAP_SIZE as f64;

        let longitude = ((x - LONG_MIN) / MAP_SIZE_D) * MAP_SIZE as f64;
        let latitude = ((y - LAT_MAX) / (-MAP_SIZE_D)) * MAP_SIZE as f64;
        (longitude, latitude)
    }

    //     pub fn smooth_map(self) -> Map {
    //         let mut stdout = stdout();
    //         let rows: Vec<Vec<_>> = self
    //             .tiles
    //             .into_iter()
    //             .map(|row| row.iter().map(|tile| tile.height).collect())
    //             .collect();
    //         println!("Smoothing cliffs");

    //         let mut smooth_rows: Vec<Vec<i64>> = Vec::new();
    //         for i in 0..MAP_SIZE {
    //             smooth_rows.push(Vec::new());
    //             for ii in 0..MAP_SIZE {
    //                 if rows[i as usize][ii as usize] < 100 {
    //                     let mut x_search = 1;
    //                     let mut y_search = 1;
    //                     'smoothLoop: loop {
    //                         for x in -(x_search)..(x_search) {
    //                             for y in [-y_search, y_search] {
    //                                 if (ii + x) < 0
    //                                     || (i + y) < 0
    //                                     || (ii + x) >= MAP_SIZE
    //                                     || (i + y) >= MAP_SIZE
    //                                 {
    //                                     continue;
    //                                 }

    //                                 if rows[(i + y) as usize][(ii + x) as usize] > 0 {
    //                                     let dampening = x_search as f64 / 10.0;
    //                                     let dampening = if dampening < 1.0 { 1.0 } else { dampening };
    //                                     smooth_rows[i as usize].push(
    //                                         ((rows[(i + y) as usize][(ii + x) as usize]) as f64
    //                                             / dampening)
    //                                             as i64,
    //                                     );
    //                                     break 'smoothLoop;
    //                                 }
    //                             }
    //                         }
    //                         for y in -(y_search)..(y_search) {
    //                             for x in [-x_search, x_search] {
    //                                 if (ii + x) < 0
    //                                     || (i + y) < 0
    //                                     || (ii + x) >= MAP_SIZE
    //                                     || (i + y) >= MAP_SIZE
    //                                 {
    //                                     continue;
    //                                 }

    //                                 if rows[(i + y) as usize][(ii + x) as usize] > 0 {
    //                                     let dampening = x_search as f64 / 10.0;
    //                                     let dampening = if dampening < 1.0 { 1.0 } else { dampening };
    //                                     smooth_rows[i as usize].push(
    //                                         ((rows[(i + y) as usize][(ii + x) as usize]) as f64
    //                                             / dampening)
    //                                             as i64,
    //                                     );
    //                                     break 'smoothLoop;
    //                                 }
    //                             }
    //                         }

    //                         x_search += 1;
    //                         y_search += 1;
    //                         if x_search > 20 {
    //                             smooth_rows[i as usize].push(rows[i as usize][ii as usize]);
    //                             break 'smoothLoop;
    //                         }
    //                     }
    //                 } else {
    //                     smooth_rows[i as usize].push(rows[i as usize][ii as usize]);
    //                 }
    //                 if ii % 10 == 0 {
    //                     let progress =
    //                         i as f64 / MAP_SIZE as f64 + (ii as f64 / (MAP_SIZE as f64).powf(2.0));
    //                     ui::print_progress_bar(progress);
    //                     stdout.flush().unwrap();
    //                 }
    //             }
    //         }
    //         ui::print_progress_bar_completed();

    //         Map {
    //             tiles: smooth_rows
    //                 .into_iter()
    //                 .map(|row| row.into_iter().map(|h| MapTile { height: h }).collect())
    //                 .collect(),
    //         }
    //     }

    //     pub fn blur_dark_spots(self) -> Map {
    //         let mut stdout = stdout();
    //         let rows: Vec<Vec<_>> = self
    //             .tiles
    //             .into_iter()
    //             .map(|row| row.iter().map(|tile| tile.height).collect())
    //             .collect();
    //         println!("Blurring dark spots");

    //         let mut smooth_rows: Vec<Vec<i64>> = Vec::new();
    //         for i in 0..MAP_SIZE {
    //             smooth_rows.push(Vec::new());
    //             for ii in 0..MAP_SIZE {
    //                 if rows[i as usize][ii as usize] < 50000 {
    //                     let mut sum = 0;
    //                     for x in -8..8 {
    //                         for y in -8..8 {
    //                             if (ii + x) < 0
    //                                 || (i + y) < 0
    //                                 || (ii + x) >= MAP_SIZE
    //                                 || (i + y) >= MAP_SIZE
    //                             {
    //                                 continue;
    //                             }

    //                             sum += rows[(i + y) as usize][(ii + x) as usize];
    //                         }
    //                     }
    //                     smooth_rows[i as usize].push(sum / 16_i64.pow(2))
    //                 } else {
    //                     smooth_rows[i as usize].push(rows[i as usize][ii as usize]);
    //                 }
    //                 if ii % 10 == 0 {
    //                     let progress =
    //                         i as f64 / MAP_SIZE as f64 + (ii as f64 / (MAP_SIZE as f64).powf(2.0));
    //                     ui::print_progress_bar(progress);
    //                     stdout.flush().unwrap();
    //                 }
    //             }
    //         }
    //         ui::print_progress_bar_completed();

    //         Map {
    //             tiles: smooth_rows
    //                 .into_iter()
    //                 .map(|row| row.into_iter().map(|h| MapTile { height: h }).collect())
    //                 .collect(),
    //         }
    //     }
}

fn build_height_map(bbox: &BboxTree<HeightPoint>) -> Vec<Vec<i64>> {
    println!("Building height map");
    let mut stdout = stdout();
    let mut rows = Vec::new();
    // let average_height = bbox.average_height();
    for y in 0..MAP_SIZE {
        let mut row = Vec::new();
        for x in 0..MAP_SIZE {
            // 61.4559276,23.6617841 61.4982935,23.7746761
            let longitude = LONG_MIN + MAP_SIZE_D * (x as f64 / MAP_SIZE as f64);
            let latitude = LAT_MAX - MAP_SIZE_D * (y as f64 / MAP_SIZE as f64);
            // let longitude = 23.7746761 - 0.001 + 0.002 * (x as f64 / MAP_SIZE as f64);
            // let latitude = 61.4982935 + 0.001 - 0.002 * (y as f64 / MAP_SIZE as f64);
            let searchArea = 0.00003;
            let searchBox = &bbox::Bbox {
                a: Point {
                    x: longitude - searchArea,
                    y: latitude - searchArea,
                },
                b: Point {
                    x: longitude + searchArea,
                    y: latitude + searchArea,
                },
            };
            let found: Vec<_> = bbox
                .find_boxes_overlapping(searchBox)
                .iter()
                .flat_map(|b| {
                    if let BboxTree::Leaf { items, .. } = b {
                        items.clone()
                    } else {
                        Vec::new()
                    }
                })
                .collect();
            if found.len() > 0 {
                let average = found
                    .iter()
                    .map(|h| h.height)
                    .map(|x| x as i64)
                    .sum::<i64>()
                    / found.len() as i64;
                row.push(average);
            } else {
                row.push(0);
            }

            if x % 10 == 0 {
                let progress =
                    y as f64 / MAP_SIZE as f64 + (x as f64 / (MAP_SIZE as f64).powf(2.0));
                ui::print_progress_bar(progress);
                stdout.flush().unwrap();
            }
        }
        rows.push(row);
    }
    ui::print_progress_bar_completed();

    println!(
        "Wrote {} points, average height {:.2}",
        MAP_SIZE.pow(2),
        rows.iter().flat_map(|r| r).sum::<i64>() as f64 / (MAP_SIZE as f64).powf(2.0)
    );
    println!(
        "with max height of {}",
        rows.iter().flat_map(|r| r).max().unwrap()
    );
    println!("Had {} data points", bbox.count_points());
    println!("with max height of {}", bbox.max_height());

    rows
}

fn build_class_map(
    bbox: &BboxTree<ClassPoint>,
    lakes: &Vec<Polygon>,
    roads: &Vec<LineSegment>,
    buildings: &Vec<Polygon>,
) -> Vec<Vec<MapTileClass>> {
    println!("Building class map");
    let mut stdout = stdout();
    let mut rows = Vec::new();

    let mut shapeBox: BboxTree<Shape> = BboxTree::Leaf {
        bbox: bbox::Bbox {
            // a: bbox::Point { x: 23.37, y: 61.19 },
            // b: bbox::Point { x: 23.47, y: 61.39 }
            a: Point { x: 23.0, y: 60.0 },
            b: Point { x: 24.0, y: 62.0 },
        },
        items: Vec::new(),
    };

    for road in roads {
        shapeBox = shapeBox.add_item(&Shape::LINE_SEGMENT(road.clone(), MapTileClass::ROAD), 0);
    }

    for building in buildings {
        shapeBox = shapeBox.add_item(&Shape::POLYGON(building.clone(), MapTileClass::BUILDING), 0);
    }

    // let average_height = bbox.average_height();
    for y in 0..MAP_SIZE {
        let mut row = Vec::new();
        for x in 0..MAP_SIZE {
            // 61.4559276,23.6617841 61.4982935,23.7746761
            let longitude = LONG_MIN + MAP_SIZE_D * (x as f64 / MAP_SIZE as f64);
            let latitude = LAT_MAX - MAP_SIZE_D * (y as f64 / MAP_SIZE as f64);
            // let longitude = 23.7746761 - 0.001 + 0.002 * (x as f64 / MAP_SIZE as f64);
            // let latitude = 61.4982935 + 0.001 - 0.002 * (y as f64 / MAP_SIZE as f64);

            let searchArea = 0.00001;
            let searchBox = &bbox::Bbox {
                a: Point {
                    x: longitude - searchArea,
                    y: latitude - searchArea,
                },
                b: Point {
                    x: longitude + searchArea,
                    y: latitude + searchArea,
                },
            };
            let found = bbox.find_boxes_overlapping(searchBox);
            let roads = shapeBox
                .find_boxes_overlapping(searchBox)
                .iter()
                .filter_map(|x| {
                    if let bbox::BboxTree::Leaf { items, .. } = x {
                        Some(items.iter().filter_map(|x| {
                            if let Shape::LINE_SEGMENT(l, MapTileClass::ROAD) = x {
                                Some(l)
                            } else {
                                None
                            }
                        }))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect::<Vec<_>>();
            let buildings = shapeBox
                .find_boxes_overlapping(searchBox)
                .iter()
                .filter_map(|x| {
                    if let bbox::BboxTree::Leaf { items, .. } = x {
                        Some(items.iter().filter_map(|x| {
                            if let Shape::POLYGON(l, MapTileClass::BUILDING) = x {
                                Some(l)
                            } else {
                                None
                            }
                        }))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect::<Vec<_>>();
            if lakes
                .iter()
                .find(|l| {
                    l.contains_point(&Point {
                        x: longitude,
                        y: latitude,
                    })
                })
                .is_some()
            {
                row.push(MapTileClass::WATER);
            } else if buildings
                .iter()
                .find(|r| {
                    r.contains_point(&Point {
                        x: longitude,
                        y: latitude,
                    })
                })
                .is_some()
            {
                row.push(MapTileClass::BUILDING);
            } else if roads
                .iter()
                .find(|r| {
                    r.distance_squared_to_point(&Point {
                        x: longitude,
                        y: latitude,
                    }) < 0.00001_f64.powf(2.0)
                })
                .is_some()
            {
                row.push(MapTileClass::ROAD);
            } else {
                let point = &Point {
                    x: longitude,
                    y: latitude,
                };
                let items = found.iter().flat_map(|x| {
                    if let bbox::BboxTree::Leaf { items, .. } = x {
                        items.clone()
                    } else {
                        Vec::new()
                    }
                });

                let mut class = items
                    .filter(|p| {
                        p.coords.distance_squared(&Point {
                            x: longitude,
                            y: latitude,
                        }) < 0.0000001
                    })
                    .collect::<Vec<_>>();
                class.sort_by(|a, b| {
                    a.coords
                        .distance_squared(point)
                        .partial_cmp(&b.coords.distance_squared(point))
                        .unwrap()
                });
                let class = class.first();
                row.push(
                    class
                        .map(|c| &c.class)
                        .unwrap_or(&MapTileClass::EMPTY)
                        .clone(),
                );
            }

            if x % 10 == 0 {
                let progress =
                    y as f64 / MAP_SIZE as f64 + (x as f64 / (MAP_SIZE as f64).powf(2.0));
                ui::print_progress_bar(progress);
                stdout.flush().unwrap();
            }
        }
        rows.push(row);
    }
    ui::print_progress_bar_completed();

    rows
}

fn finnish_coords_to_global(N: f64, E: f64) -> Point {
    let f = 1.0 / 298.257222101; // Ellipsoidin litistyssuhde
    let a = 6378137.0; // Isoakselin puolikas
    let lambda_nolla = 0.471238898; // Keskimeridiaani (rad), 27 astetta
    let k_nolla = 0.9996; // Mittakaavakerroin
    let E_nolla = 500000.0; // It�koordinaatti

    // Kaavat
    let n: f64 = f / (2.0 - f);
    let A1 = (a / (1.0 + n)) * (1.0 + (n.powf(2.0) / 4.0) + (n.powf(4.0) / 64.0));
    let e_toiseen = (2.0 * f) - f.powf(2.0);
    let h1 = (1.0 / 2.0) * n - (2.0 / 3.0) * n.powf(2.0) + (37.0 / 96.0) * n.powf(3.0)
        - (1.0 / 360.0) * n.powf(4.0);
    let h2 =
        (1.0 / 48.0) * n.powf(2.0) + (1.0 / 15.0) * n.powf(3.0) - (437.0 / 1440.0) * n.powf(4.0);
    let h3 = (17.0 / 480.0) * n.powf(3.0) - (37.0 / 840.0) * n.powf(4.0);
    let h4 = (4397.0 / 161280.0) * n.powf(4.0);
    let zeeta = N / (A1 * k_nolla);
    let eeta = (E - E_nolla) / (A1 * k_nolla);
    let zeeta1_pilkku = h1 * (2.0 * zeeta).sin() * (2.0 * eeta).cosh();
    let zeeta2_pilkku = h2 * (4.0 * zeeta).sin() * (4.0 * eeta).cosh();
    let zeeta3_pilkku = h3 * (6.0 * zeeta).sin() * (6.0 * eeta).cosh();
    let zeeta4_pilkku = h4 * (8.0 * zeeta).sin() * (8.0 * eeta).cosh();
    let eeta1_pilkku = h1 * (2.0 * zeeta).cos() * (2.0 * eeta).sinh();
    let eeta2_pilkku = h2 * (4.0 * zeeta).cos() * (4.0 * eeta).sinh();
    let eeta3_pilkku = h3 * (6.0 * zeeta).cos() * (6.0 * eeta).sinh();
    let eeta4_pilkku = h4 * (8.0 * zeeta).cos() * (8.0 * eeta).sinh();
    let zeeta_pilkku = zeeta - (zeeta1_pilkku + zeeta2_pilkku + zeeta3_pilkku + zeeta4_pilkku);
    let eeta_pilkku = eeta - (eeta1_pilkku + eeta2_pilkku + eeta3_pilkku + eeta4_pilkku);
    let beeta = ((1.0 / (eeta_pilkku).cosh() * (zeeta_pilkku).sin()) as f64).asin();
    let l = ((eeta_pilkku).tanh() / ((beeta).cos())).asin();
    let Q = ((beeta).tan()).asinh();
    let mut Q_pilkku = Q + (e_toiseen).sqrt() * ((e_toiseen).sqrt() * (Q).tanh()).atanh();

    for kierros in 1..5 {
        Q_pilkku = Q + (e_toiseen).sqrt() * ((e_toiseen).sqrt().atanh() * (Q_pilkku).tanh());
    }

    // Tulos radiaaneina
    let fii = ((Q_pilkku).sinh()).atan();
    let lambda = lambda_nolla + l;

    // Tulos asteina
    let fii = fii * 57.2957795;
    let lambda = lambda * 57.2957795;

    // let array = array ('lev' => $fii, 'pit' => $lambda);
    Point { x: lambda, y: fii }
}
