use super::point::Point;

#[derive(Debug, Clone)]
pub struct Bbox {
    pub a: Point,
    pub b: Point,
}

impl Bbox {
    pub fn contains(self: &Bbox, point: &Point) -> bool {
        self.a.x <= point.x && point.x <= self.b.x && self.a.y <= point.y && point.y <= self.b.y
    }

    pub fn overlaps(self: &Bbox, other: &Bbox) -> bool {
        !(self.b.x < other.a.x
            || self.a.x > other.b.x
            || self.b.y < other.a.y
            || self.a.y > other.b.y)
    }

    pub fn split(self: &Bbox) -> (Bbox, Bbox, Bbox, Bbox) {
        let half_width = (self.b.x - self.a.x) / 2.0;
        let half_height = (self.b.y - self.a.y) / 2.0;

        (
            Bbox {
                a: Point {
                    x: self.a.x,
                    y: self.a.y,
                },
                b: Point {
                    x: self.a.x + half_width,
                    y: self.a.y + half_height,
                },
            },
            Bbox {
                a: Point {
                    x: self.a.x + half_width,
                    y: self.a.y,
                },
                b: Point {
                    x: self.b.x,
                    y: self.a.y + half_height,
                },
            },
            Bbox {
                a: Point {
                    x: self.a.x,
                    y: self.a.y + half_height,
                },
                b: Point {
                    x: self.a.x + half_width,
                    y: self.b.y,
                },
            },
            Bbox {
                a: Point {
                    x: self.a.x + half_width,
                    y: self.a.y + half_height,
                },
                b: Point {
                    x: self.b.x,
                    y: self.b.y,
                },
            },
        )
    }
}

pub trait HasCoords {
    fn fits_into(&self, bbox: &Bbox) -> bool;
}

#[derive(Debug)]
pub enum BboxTree<T: HasCoords + Clone> {
    Interior {
        bbox: Bbox,
        children: (
            Box<BboxTree<T>>,
            Box<BboxTree<T>>,
            Box<BboxTree<T>>,
            Box<BboxTree<T>>,
        ),
    },
    Leaf {
        bbox: Bbox,
        items: Vec<T>,
    },
}

fn add_item_if_fits<T: HasCoords + Clone>(
    node: BboxTree<T>,
    item: &T,
    depth: usize,
) -> BboxTree<T> {
    match node {
        BboxTree::Interior { ref bbox, .. } | BboxTree::Leaf { ref bbox, .. } => {
            if item.fits_into(&bbox) {
                node.add_item(item, depth)
            } else {
                node
            }
        }
    }
}

impl<T: HasCoords + Clone> BboxTree<T> {
    pub fn add_item(mut self: BboxTree<T>, item: &T, depth: usize) -> BboxTree<T> {
        match self {
            BboxTree::Interior { children, bbox } => BboxTree::Interior {
                bbox,
                children: (
                    Box::new(add_item_if_fits(*children.0, &item, depth + 1)),
                    Box::new(add_item_if_fits(*children.1, &item, depth + 1)),
                    Box::new(add_item_if_fits(*children.2, &item, depth + 1)),
                    Box::new(add_item_if_fits(*children.3, &item, depth + 1)),
                ),
            },
            BboxTree::Leaf {
                ref mut items,
                bbox,
            } if items.len() >= 10 && depth < 40 => {
                items.push(item.clone());

                let split = bbox.split();
                let mut new_interior: BboxTree<T> = BboxTree::Interior {
                    bbox,
                    children: (
                        Box::new(BboxTree::Leaf {
                            bbox: split.0,
                            items: Vec::new(),
                        }),
                        Box::new(BboxTree::Leaf {
                            bbox: split.1,
                            items: Vec::new(),
                        }),
                        Box::new(BboxTree::Leaf {
                            bbox: split.2,
                            items: Vec::new(),
                        }),
                        Box::new(BboxTree::Leaf {
                            bbox: split.3,
                            items: Vec::new(),
                        }),
                    ),
                };

                for item in items {
                    new_interior = new_interior.add_item(item, depth + 1);
                }

                new_interior
            }
            BboxTree::Leaf { ref mut items, .. } => {
                items.push(item.clone());
                self
            }
        }
    }

    pub fn find_boxes_overlapping(self: &BboxTree<T>, other: &Bbox) -> Vec<&BboxTree<T>> {
        match self {
            BboxTree::Interior {
                ref bbox,
                ref children,
            } => {
                if bbox.overlaps(other) {
                    let mut v = Vec::new();
                    v.append(&mut (*children.0).find_boxes_overlapping(other));
                    v.append(&mut (*children.1).find_boxes_overlapping(other));
                    v.append(&mut (*children.2).find_boxes_overlapping(other));
                    v.append(&mut (*children.3).find_boxes_overlapping(other));
                    return v;
                }

                Vec::new()
            }
            BboxTree::Leaf { ref bbox, .. } => {
                if bbox.overlaps(other) {
                    return vec![self];
                }

                Vec::new()
            }
        }
    }

    pub fn find_box_for(self: &BboxTree<T>, point: &Point) -> Option<&BboxTree<T>> {
        match self {
            BboxTree::Interior {
                ref bbox,
                ref children,
            } => {
                if bbox.contains(point) {
                    return [
                        (*children.0).find_box_for(point),
                        (*children.1).find_box_for(point),
                        (*children.2).find_box_for(point),
                        (*children.3).find_box_for(point),
                    ]
                    .into_iter()
                    .filter_map(|x| x)
                    .next();
                }

                None
            }
            BboxTree::Leaf { ref bbox, .. } => {
                if bbox.contains(point) {
                    return Some(self);
                }

                None
            }
        }
    }

    pub fn count_points(self: &BboxTree<T>) -> usize {
        match self {
            BboxTree::Interior { ref children, .. } => {
                (*children.0).count_points()
                    + (*children.1).count_points()
                    + (*children.2).count_points()
                    + (*children.3).count_points()
            }
            BboxTree::Leaf { ref items, .. } => items.len(),
        }
    }
}
