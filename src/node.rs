use vec::Vec3;
use core::num::Zero;
use rayon::prelude::*;
use ::THETA_SQUARED;

#[derive(Debug)]
pub struct Body {
    pub x: Vec3<f64>,
    pub v: Vec3<f64>,
    pub a: Vec3<f64>,
    pub m: f64,
}

#[derive(Debug)]
pub struct Section {
    center: Vec3<f64>,
    com: Vec3<f64>,
    pub total_mass: f64,
    width: f64,
    sub: Option<Box<[Option<Section>; 8]>>,
}

impl Section {
    pub fn compute(&self, bodies: &mut [Body]) {
        bodies.par_iter_mut().for_each(|b| {
            b.a = Vec3::zero();
            self.attract(b);
        })
    }

    fn attract(&self, body: &mut Body) {
        for sect in self.sub.as_ref().unwrap().iter() {
            if let Some(ref s) = *sect {
                if s.com != body.x {
                    let dx = s.com - body.x;
                    let inv_dist_sq = 1.0 / dx.dot(dx);

                    if s.width * s.width * inv_dist_sq < THETA_SQUARED {
                        body.a += dx * (self.total_mass * inv_dist_sq * inv_dist_sq.sqrt());
                    } else {
                        s.attract(body);
                    }
                }
            }
        }
    }

    pub fn add(&mut self, point: Vec3<f64>, mass: f64) {
        let pos = self.position(point);
        let n = &mut self.sub.as_mut().unwrap()[pos];

        if let Some(ref mut sect) = *n {
            if !sect.sub.is_some() {
                sect.width = self.width / 2.0;
                let (old_point, old_mass) = (sect.com, sect.total_mass);
                sect.com = Vec3::zero();
                sect.total_mass = 0.0;

                let offset = Section::offset(self.center, old_point, sect.width);
                sect.center = self.center + offset;

                sect.sub = Some(box [None, None, None, None, None, None, None, None]);
                sect.add(old_point, old_mass);
            }
            sect.add(point, mass);
        } else {
            *n = Some(Section {
                com: point,
                total_mass: mass,
                center: Vec3::zero(),
                width: 0.0,
                sub: None,
            });
        }
    }

    pub fn aggregate(&mut self) {
        for sect in self.sub.as_mut().unwrap().iter_mut() {
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

    fn new(old_center: Vec3<f64>, width: f64, offset: Vec3<f64>) -> Self {
        Section {
            center: old_center + offset,
            width: width,
            sub: Some(box [None, None, None, None, None, None, None, None]),
            com: Vec3::zero(),
            total_mass: 0.0,
        }
    }

    pub fn containing(bodies: &[Body]) -> Self {
        Section::new(Vec3(0.5, 0.5, 0.5), 0.5, Vec3::zero())
    }

    fn position(&self, point: Vec3<f64>) -> usize {
        (if point.0 > self.center.0 { 1 } else { 0 } +
         if point.1 > self.center.1 { 2 } else { 0 } +
         if point.2 > self.center.2 { 4 } else { 0 })
    }

    fn offset(center: Vec3<f64>, point: Vec3<f64>, dist: f64) -> Vec3<f64> {
        Vec3(if point.0 > center.0 { dist } else { -dist },
             if point.1 > center.1 { dist } else { -dist },
             if point.2 > center.2 { dist } else { -dist })
    }

    pub fn parallel_add(&mut self, bodies: &[Body]) {
        let new_width = self.width / 2.0;
        let offset = [-new_width, new_width];
        let mut children = Vec::with_capacity(8);

        for &k in &offset {
            for &j in &offset {
                for &i in &offset {
                    children.push(Section::new(self.center, new_width, Vec3(i,j,k)));
                }
            }
        }

        let mut parent_children = [None, None, None, None, None, None, None, None];

        self.total_mass = children.into_par_iter().zip(parent_children.par_iter_mut()).weight_max().enumerate().map(|(i, (mut node, pn))| {
            for body in bodies {
                if i == self.position(body.x) {
                    node.add(body.x, body.m);
                }
            }
            node.aggregate();
            let m = node.total_mass;
            *pn = Some(node);
            m
        }).sum();

        self.sub = Some(box parent_children);
    }
}
