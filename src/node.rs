use ::{DT, THETA_SQUARED};
use vec::Vec2;

use std::num::Zero;
use rayon::prelude::*;
use rand::random;
use itertools::Itertools;

#[derive(Debug)]
pub struct Body {
    pub x: Vec2<f64>,
    pub v: Vec2<f64>,
    pub a: Vec2<f64>,
    pub m: f64,
}

impl Body {
    pub fn generate(num_bodies: usize) -> Vec<Body> {
        (0..num_bodies).map(|_|
            Body {
                x: Vec2(random(), random()),
                v: Vec2(0.0, 0.0),
                a: Vec2::zero(),
                m: 1.0,
            }
        ).collect()
    }

    pub fn advance(&mut self) {
        self.x += (self.v + DT / 2.0 * self.a) * DT;
        self.v += self.a * DT;
        self.a = Vec2::zero();
    }
}

type SubNodes = [Option<Section>; 4];
type Children = Option<Box<SubNodes>>;

/// This macro is needed because calling &mut self captures all of self as mutable
/// In the future, we should be able to selectively borrow sub fields of a struct.
macro_rules! mut_children {
    ($e:expr) => ($e.sub.as_mut().unwrap());
}

#[derive(Debug)]
pub struct Section {
    center: Vec2<f64>,
    com: Vec2<f64>,
    pub total_mass: f64,
    width: f64,
    sub: Children,
}

impl Section {
    fn new(old_center: Vec2<f64>, width: f64, offset: Vec2<f64>) -> Self {
        Section {
            center: old_center + offset,
            width: width,
            sub: Some(box [None, None, None, None]),
            com: Vec2::zero(),
            total_mass: 0.0,
        }
    }

    pub fn containing(bodies: &[Body]) -> Self {
        let p = bodies[0].x;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (p.0, p.1, p.0, p.1);

        for body in bodies {
            if min_x > body.x.0 { min_x = body.x.0 }
            if min_y > body.x.1 { min_y = body.x.1 }
            if max_x < body.x.0 { max_x = body.x.0 }
            if max_y < body.x.1 { max_y = body.x.1 }
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

    fn attract(&self, body: &mut Body) {
        for sect in self.children().iter() {
            if let Some(ref s) = *sect {
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
    }

    pub fn aggregate(&mut self) {
        for sect in mut_children!(self).iter_mut() {
            if let Some(ref mut sect) = *sect {
                if sect.sub.is_some() {
                    sect.aggregate();
                }
                self.com += sect.com * sect.total_mass;
                self.total_mass += sect.total_mass;
            }
        }
        self.com /= self.total_mass;
    }

    fn position(&self, point: Vec2<f64>) -> usize {
        (if point.0 > self.center.0 { 1 } else { 0 } +
         if point.1 > self.center.1 { 2 } else { 0 })
    }

    fn offset(center: Vec2<f64>, point: Vec2<f64>, dist: f64) -> Vec2<f64> {
        Vec2(if point.0 > center.0 { dist } else { -dist },
             if point.1 > center.1 { dist } else { -dist })
    }

    fn density(&self) -> f64 {
        self.total_mass / self.width / self.width
    }

    fn children(&self) -> &SubNodes {
        self.sub.as_ref().unwrap()
    }

    pub fn render(&self, total: f64) -> Vec<(Vec2<f64>, f64, f64)> {
        if self.total_mass / total < 1.0 / 10000.0 {
            let upp_left = self.center - (Vec2(1.0, 1.0) * self.width);
            vec![(upp_left, self.width * 2.0, self.total_mass / total)]
        } else {
            self.children().iter().flatten().map(|node|
                node.render(total)
            ).flatten().collect()
        }
    }

    pub fn add(&mut self, point: Vec2<f64>, mass: f64) {
        let pos = self.position(point);
        let n = &mut mut_children!(self)[pos];

        if let Some(ref mut sect) = *n {
            if !sect.sub.is_some() {
                sect.width = self.width / 2.0;
                let (old_point, old_mass) = (sect.com, sect.total_mass);
                sect.com = Vec2::zero();
                sect.total_mass = 0.0;

                let offset = Section::offset(self.center, old_point, sect.width);
                sect.center = self.center + offset;

                sect.sub = Some(box [None, None, None, None]);
                sect.add(old_point, old_mass);
            }
            sect.add(point, mass);
        } else {
            *n = Some(Section {
                com: point,
                total_mass: mass,
                center: Vec2::zero(),
                width: 0.0,
                sub: None,
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
                children.push(Section::new(self.center, new_width, Vec2(i,j)));
            }
        }

        let mut parent_children = [None, None, None, None];

        self.total_mass = children.into_par_iter().weight_max()
                            .zip(parent_children.par_iter_mut())
                            .enumerate().map(|(i, (mut node, pn))| {
            for body in bodies {
                if i == self.position(body.x) {
                    node.add(body.x, body.m);
                }
            }
            node.aggregate(); // Aggregate in parallel
            let m = node.total_mass;
            *pn = Some(node);
            m
        }).sum();

        self.sub = Some(box parent_children);
    }
}
