use super::{DT, THETA_SQUARED};
use vec::Vec2;

use rand::random;
use std::num::Zero;
use std::ops::{Deref, DerefMut};
use std::slice::{Iter, IterMut};
use rayon::prelude::*;
use itertools::{Flatten, Itertools};

#[derive(Debug)]
pub struct Body {
    pub x: Vector,
    pub v: Vector,
    pub a: Vector,
    pub m: f64,
}

impl Body {
    pub fn generate_collision(num_bodies: usize) -> Vec<Body> {
        (0..num_bodies / 2)
            .map(|_| {
                Body {
                    x: Vec2(random::<f64>() / 2.0, random::<f64>() / 2.0),
                    v: Vec2(0.0, 0.0),
                    a: Vec2::zero(),
                    m: 1.0,
                }
            })
            .chain((0..num_bodies / 2).map(|_| {
                Body {
                    x: Vec2((1.0 + random::<f64>()) / 2.0, (1.0 + random::<f64>()) / 2.0),
                    v: Vec2(0.0, 0.0),
                    a: Vec2::zero(),
                    m: 1.0,
                }
            }))
            .collect()
    }

    pub fn generate(num_bodies: usize) -> Vec<Body> {
        (0..num_bodies)
            .map(|_| {
                Body {
                    x: Vec2(random(), random()),
                    v: Vec2(0.0, 0.0),
                    a: Vec2::zero(),
                    m: 1.0,
                }
            })
            .collect()
    }

    pub fn advance(&mut self) {
        self.x += (self.v + DT / 2.0 * self.a) * DT;
        self.v += self.a * DT;
        self.a = Vec2::zero();
    }
}

pub type Elem = f64;
pub type Vector = Vec2<Elem>;
type SubNodes = [Option<Section>; 4];
type PointerChildren = Option<Box<SubNodes>>;

#[derive(Debug)]
struct Children(PointerChildren);

impl Deref for Children {
    type Target = PointerChildren;

    fn deref(&self) -> &PointerChildren {
        &self.0
    }
}

impl DerefMut for Children {
    fn deref_mut(&mut self) -> &mut PointerChildren {
        &mut self.0
    }
}

impl<'a> Children {
    fn children(&self) -> &SubNodes {
        self.as_ref().unwrap()
    }

    fn mut_children(&mut self) -> &mut SubNodes {
        self.as_mut().unwrap()
    }

    fn all_children(&'a self) -> Flatten<Iter<'a, Option<Section>>> {
        self.children().iter().flatten()
    }

    fn mut_all_children(&'a mut self) -> Flatten<IterMut<'a, Option<Section>>> {
        self.mut_children().iter_mut().flatten()
    }

    fn has_node_children(&self) -> bool {
        self.all_children().any(Section::is_node)
    }
}

#[derive(Debug)]
pub struct Section {
    center: Vector,
    com: Vector,
    pub total_mass: f64,
    width: f64,
    sub: Children,
}

impl Section {
    fn new(old_center: Vector, width: f64, offset: Vector) -> Self {
        Section {
            center: old_center + offset,
            width: width,
            sub: Children(Some(box [None, None, None, None])),
            com: Vec2::zero(),
            total_mass: 0.0,
        }
    }

    pub fn containing(bodies: &[Body]) -> Self {
        let p = bodies[0].x;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (p.0, p.1, p.0, p.1);

        for body in bodies {
            min_x = min_x.min(body.x.0);
            min_y = min_y.min(body.x.1);
            max_x = max_x.max(body.x.0);
            max_y = max_y.max(body.x.1);
        }

        let lower_left  = Vec2(min_x, min_y);
        let upper_right = Vec2(max_x, max_y);

        let center = (lower_left + upper_right) / 2.0;
        let size = (upper_right - lower_left).inf_norm() / 2.0;

        Section::new(center, size, Vec2::zero())
    }

    pub fn compute(&self, bodies: &mut [Body]) {
        bodies.par_iter_mut().for_each(|b| self.attract(b))
    }

    fn is_node(&self) -> bool {
        self.sub.is_some()
    }

    fn attract(&self, body: &mut Body) {
        for s in self.sub.all_children() {
            if s.com != body.x {
                let dx = s.com - body.x;
                let inv_dist_sq = 1.0 / (dx.dot(dx) + 0.0001);

                if s.width * s.width * inv_dist_sq < THETA_SQUARED {
                    body.a += dx * (self.total_mass * inv_dist_sq * inv_dist_sq.sqrt());
                } else {
                    s.attract(body);
                }
            }
        }
    }

    pub fn aggregate(&mut self) {
        for sect in self.sub.mut_all_children() {
            if sect.is_node() {
                sect.aggregate();
            }
            self.com += sect.com * sect.total_mass;
            self.total_mass += sect.total_mass;
        }
        self.com /= self.total_mass;
    }

    fn position(&self, point: Vector) -> usize {
        (if point.0 > self.center.0 { 1 } else { 0 } +
         if point.1 > self.center.1 { 2 } else { 0 })
    }

    fn offset(center: Vector, point: Vector, dist: f64) -> Vector {
        Vec2(if point.0 > center.0 { dist } else { -dist },
             if point.1 > center.1 { dist } else { -dist })
    }

    pub fn density(&self) -> f64 {
        self.total_mass / self.width / self.width / 4.0
    }

    pub fn upper_left(&self) -> Vector {
        self.center - (Vec2(1.0, 1.0) * self.width)
    }

    pub fn render(&self, reference: f64, total: f64) -> Vec<(Vector, f64, f64)> {
        if self.total_mass / total < 1E-4 || !self.sub.has_node_children() {
            vec![(self.upper_left(), self.width * 2.0, (self.density() / reference).ln() / 5.0)]
        } else {
            self.sub.all_children()
                .map(|node| node.render(reference, total))
                .flatten().collect()
        }
    }

    pub fn add(&mut self, point: Vector, mass: f64) {
        let pos = self.position(point);
        let n = &mut self.sub.mut_children()[pos];

        if let Some(ref mut sect) = *n {
            if !sect.sub.is_some() {
                sect.width = self.width / 2.0;
                let (old_point, old_mass) = (sect.com, sect.total_mass);
                sect.com = Vec2::zero();
                sect.total_mass = 0.0;

                let offset = Section::offset(self.center, old_point, sect.width);
                sect.center = self.center + offset;

                sect.sub = Children(Some(box [None, None, None, None]));
                sect.add(old_point, old_mass);
            }
            sect.add(point, mass);
        } else {
            *n = Some(Section {
                com: point,
                total_mass: mass,
                center: Vec2::zero(),
                width: 0.0,
                sub: Children(None),
            });
        }
    }

    /// TODO: fix this
    pub fn parallel_add(&mut self, bodies: &[Body]) {
        let new_width = self.width / 2.0;
        let offset = [-new_width, new_width];
        let mut children = Vec::with_capacity(8);

        for &j in &offset {
            for &i in &offset {
                children.push(Section::new(self.center, new_width, Vec2(i, j)));
            }
        }

        let mut parent_children = [None, None, None, None];

        self.total_mass = children.into_par_iter()
            .weight_max()
            .zip(parent_children.par_iter_mut())
            .enumerate()
            .map(|(i, (mut node, pn))| {
                for body in bodies {
                    if i == self.position(body.x) {
                        node.add(body.x, body.m);
                    }
                }
                node.aggregate(); // Aggregate in parallel
                let m = node.total_mass;
                *pn = Some(node);
                m
            })
            .sum();

        self.sub = Children(Some(box parent_children));
    }
}
