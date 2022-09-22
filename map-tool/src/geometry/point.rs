use std::ops;

use super::bbox;

#[derive(Debug, Clone)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn length_squared(&self) -> f64 {
        self.x.powf(2.0) + self.y.powf(2.0)
    }

    pub fn dot(&self, other: &Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl ops::Mul<f64> for &Vec2 {
    type Output = Vec2;
    fn mul(self, f: f64) -> Vec2 {
        Vec2 {
            x: self.x * f,
            y: self.y * f,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: -(self.x * other.z - self.z * other.x),
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0)).sqrt()
    }

    pub fn normalize(&self) -> Vec3 {
        self * (1.0 / self.magnitude())
    }
}

impl ops::Mul<f64> for &Vec3 {
    type Output = Vec3;
    fn mul(self, f: f64) -> Vec3 {
        Vec3 {
            x: self.x * f,
            y: self.y * f,
            z: self.z * f,
        }
    }
}

impl ops::Add<&Vec3> for &Vec3 {
    type Output = Vec3;
    fn add(self, v: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
}

impl ops::Sub<&Vec3> for &Vec3 {
    type Output = Vec3;
    fn sub(self, v: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x - v.x,
            y: self.y - v.y,
            z: self.z - v.z,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn distance(&self, point: &Point) -> f64 {
        ((self.x - point.x).powf(2.0) + (self.y - point.y).powf(2.0)).sqrt()
    }

    pub fn distance_squared(&self, point: &Point) -> f64 {
        (self.x - point.x).powf(2.0) + (self.y - point.y).powf(2.0)
    }
}

impl ops::Add<&Vec2> for &Point {
    type Output = Point;
    fn add(self, other: &Vec2) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub<&Point> for &Point {
    type Output = Vec2;
    fn sub(self, other: &Point) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

pub struct Line {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

#[derive(Clone)]
pub struct LineSegment {
    pub a: Point,
    pub b: Point,
}

impl LineSegment {
    pub fn intersects_segment(&self, line: &LineSegment) -> bool {
        self.intersects_line(&line.to_line()) && line.intersects_line(&self.to_line())
    }

    pub fn intersects_line(&self, line: &Line) -> bool {
        let d1 = (line.a * self.a.x) + (line.b * self.a.y) + line.c;
        let d2 = (line.a * self.b.x) + (line.b * self.b.y) + line.c;

        if d1 > 0.0 && d2 > 0.0 {
            return false;
        }
        if d1 < 0.0 && d2 < 0.0 {
            return false;
        }

        true
    }

    pub fn bbox(&self) -> bbox::Bbox {
        bbox::Bbox {
            a: Point {
                x: self.a.x.min(self.b.x),
                y: self.a.y.min(self.b.y),
            },
            b: Point {
                x: self.a.x.max(self.b.x),
                y: self.a.y.max(self.b.y),
            },
        }
    }

    pub fn length_squared(&self) -> f64 {
        self.a.distance_squared(&self.b)
    }

    pub fn distance_squared_to_point(&self, point: &Point) -> f64 {
        let l = self.length_squared();
        if l == 0.0 {
            return self.a.distance_squared(point);
        }

        let t = ((point - &self.a).dot(&(&self.b - &self.a)) / l).clamp(0.0, 1.0);
        let projection = &self.a + &(&(&self.b - &self.a) * t);
        // println!(
        //     "t: {}, l:{}, k: {}, d: {}, rrr: {}",
        //     t,
        //     l,
        //     ((&self.a - point).dot(&(point - &self.b)) / l),
        //     (point - &self.a).dot(&(&self.b - &self.a)),
        //     point.distance_squared(&projection)
        // );
        // println!("a{:?}, b{:?}", self.a, self.b);
        return point.distance_squared(&projection);
    }

    pub fn to_line(&self) -> Line {
        Line {
            a: self.b.y - self.a.y,
            b: self.a.x - self.b.x,
            c: (self.b.x * self.a.y) - (self.a.x * self.b.y),
        }
    }
}

#[derive(Clone)]
pub struct Polygon {
    vertices: Vec<Point>,
    edges: Vec<LineSegment>,
    bbox: bbox::Bbox,
}

impl Polygon {
    pub fn new(vertices: Vec<Point>) -> Polygon {
        Polygon {
            edges: edges(&vertices),
            bbox: bounding_box(&vertices),
            vertices,
        }
    }

    pub fn bbox(&self) -> &bbox::Bbox {
        &self.bbox
    }
}

impl Polygon {
    pub fn contains_point(&self, point: &Point) -> bool {
        if !self.bbox.contains(point) {
            return false;
        }

        let segment = &LineSegment {
            a: Point {
                x: self.bbox.a.x - 1.0,
                y: self.bbox.a.y - 1.0,
            },
            b: Point {
                x: point.x,
                y: point.y,
            },
        };
        let edges = &self.edges;
        let collisions = edges
            .iter()
            .filter(|e| e.intersects_segment(segment))
            .collect::<Vec<_>>()
            .len();

        collisions > 0 && collisions % 2 == 1
    }
}

fn edges(vertices: &Vec<Point>) -> Vec<LineSegment> {
    let mut edges: Vec<LineSegment> = Vec::new();
    for i in 1..vertices.len() {
        let segment = LineSegment {
            a: vertices[i - 1].clone(),
            b: vertices[i].clone(),
        };
        if edges
            .iter()
            .find(|e| e.intersects_segment(&segment))
            .is_none()
            || true
        // todo wont work
        {
            edges.push(segment);
        }
    }

    loop {
        let closing = LineSegment {
            a: vertices.last().unwrap().clone(),
            b: vertices.first().unwrap().clone(),
        };
        if edges
            .iter()
            .find(|e| e.intersects_segment(&closing))
            .is_none()
            || true
        // todo wont work
        {
            edges.push(closing);
            break;
        } else {
            println!("⚠ Can't close polygon!!!");
            if edges.pop().is_none() {
                break;
            }
        }
    }

    edges
}

fn bounding_box(vertices: &Vec<Point>) -> bbox::Bbox {
    bbox::Bbox {
        a: Point {
            x: vertices
                .iter()
                .map(|v| v.x)
                .fold(f64::INFINITY, |a, b| a.min(b)),
            y: vertices
                .iter()
                .map(|v| v.y)
                .fold(f64::INFINITY, |a, b| a.min(b)),
        },
        b: Point {
            x: vertices
                .iter()
                .map(|v| v.x)
                .fold(-f64::INFINITY, |a, b| a.max(b)),
            y: vertices
                .iter()
                .map(|v| v.y)
                .fold(-f64::INFINITY, |a, b| a.max(b)),
        },
    }
}
