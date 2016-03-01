use vec::Vec3;
use core::num::Zero;
use rayon::prelude::*;
use ::THETA;

#[derive(Debug)]
pub enum Node<'a> {
    Leaf(&'a Body),
    Internal(Option<Box<Section<'a>>>),
}

#[derive(Debug)]
pub struct Body {
    pub x: Vec3<f64>,
    pub v: Vec3<f64>,
    pub m: f64,
}

#[derive(Debug)]
pub struct Section<'a> {
    center: Vec3<f64>,
    com: Vec3<f64>,
    total_mass: f64,
    width: f64,
    sub: [Node<'a>; 8],
}

impl<'a> Section<'a> {
    pub fn containing(bodies: &[Body]) -> Self {
        Section::new(Vec3(0.5, 0.5, 0.5), 0.5, Vec3::zero())
    }

    fn contains(&self, point: Vec3<f64>) -> bool {
        let dx = self.center - point;
        dx.0.abs() < self.width &&
        dx.1.abs() < self.width &&
        dx.2.abs() < self.width
    }

    fn count(&self) -> usize {
        self.sub.iter().map(|n| match n {
            &Node::Internal(Some(box ref n)) => n.count(),
            &Node::Leaf(_) => 1,
            _ => 0,
        }).sum::<usize>() + 0
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

    fn new(old_center: Vec3<f64>, width: f64, offset: Vec3<f64>) -> Self {
        Section {
            center: old_center + offset,
            width: width,
            sub: [Node::Internal(None), Node::Internal(None), Node::Internal(None), Node::Internal(None),
                  Node::Internal(None), Node::Internal(None), Node::Internal(None), Node::Internal(None)],
            total_mass: 0.0,
            com: Vec3::zero()
        }
    }

    fn add(&mut self, body: &'a Body) {
        let sn = &mut self.sub[self.position(body.x)];
        self.total_mass += body.m;

        match sn {
            &mut Node::Leaf(leaf) => {
                let new_width = self.width / 2.0;
                let offset = Section::offset(self.center, leaf.x, new_width);
                let mut new_node = Section::new(self.center, new_width, offset);
                new_node.add(leaf);
                new_node.add(body);
                *sn = Node::Internal(Some(box new_node));
            },
            &mut Node::Internal(Some(ref mut section)) => section.add(body),
            &mut Node::Internal(None) => *sn = Node::Leaf(body),
        }
    }

    pub fn parallel_add(&mut self, bodies: &'a [Body]) {
        let new_width = self.width / 2.0;
        let offset = [-new_width, new_width];
        let mut index = 0;

        for &k in &offset {
            for &j in &offset {
                for &i in &offset {
                    self.sub[index] = Node::Internal(Some(box Section::new(self.center, new_width, Vec3(i,j,k))));
                    index += 1;
                }
            }
        }

        self.total_mass =
            self.sub.par_iter_mut().weight_max().map(|node| {
                match node {
                    &mut Node::Internal(Some(ref mut section)) => {
                        for body in bodies {
                            if section.contains(body.x) {
                                section.add(body);
                            }
                        }
                        section.total_mass
                    },
                    _ => unreachable!(),
                }
            }).sum();
    }
}

impl<'a> Node<'a> {
    pub fn compute(&self, bodies: &[Body], forces: &mut [Vec3<f64>]) {
        forces.par_iter_mut()
              .zip(bodies)
              .for_each(|(f, b)| *f = self.force(b))
    }

    fn force(&self, body: &Body) -> Vec3<f64> {
        match self {
            &Node::Leaf(leaf) =>
                if leaf as *const _ == body {
                    Vec3::zero()
                } else {
                    let (dx, inv_dist_sq, inv_dist) = leaf.x.dist(body.x);
                    dx * leaf.m * inv_dist_sq * inv_dist
                },
            &Node::Internal(Some(ref section)) => {
                    let (dx, inv_dist_sq, inv_dist) = section.com.dist(body.x);

                    if section.width * inv_dist <= THETA {
                        dx * section.total_mass * inv_dist_sq * inv_dist
                    } else {
                        section.sub.iter().map(|n| n.force(body)).sum()
                    }
                }
            _ => Vec3::zero()
        }
    }
}
