use rand::random;
use std::f64::consts::PI;
use std::num::Zero;

use vec::Vec2;
use node::Body;

pub fn galaxy(n: usize, center: Vec2<f64>, outer_radius: f64, overall: Vec2<f64>) -> Box<Iterator<Item=Body>> {
    box (0..n)
        .map(move |_| {
            let r = random::<f64>().powf(0.5) * outer_radius;
            let theta = random::<f64>() * 2.0 * PI;
            let m = 1.0;
            let area_prop = (r / outer_radius).powi(2);
            let acc = (n as f64 * m) * area_prop;
            let speed = (acc * r).sqrt();
            let vel = speed * Vec2(-theta.sin(), theta.cos());
            let offset = r * Vec2(theta.cos(), theta.sin());

            Body {
                x: center + offset,
                v: vel * 10.0 + overall,
                a: Vec2::zero(),
                m: m,
            }
        })
}

pub fn galaxy_collision(n: usize) -> Vec<Body> {
    let g1 = galaxy(n / 2, Vec2(0.3, 0.3), 0.2, Vec2(0.0, 500.0));
    let g2 = galaxy(n / 2, Vec2(0.6, 0.6), 0.2, Vec2(0.0, -500.0));
    g1.chain(g2).collect()
}

pub fn square_collision(n: usize) -> Vec<Body> {
    let s1 = (0..n / 2)
        .map(|_|
             Body {
                 x: Vec2(random::<f64>() / 2.0, random::<f64>() / 2.0),
                 v: Vec2(0.0, 0.0),
                 a: Vec2::zero(),
                 m: 1.0,
             });

    let s2 = (0..n / 2)
        .map(|_|
             Body {
                 x: Vec2((0.5 + random::<f64>()) / 2.0, (1.2 + random::<f64>()) / 2.0),
                 v: Vec2(0.0, -4000.0),
                 a: Vec2::zero(),
                 m: 1.0,
             });

    s1.chain(s2).collect()
}

pub fn simple_square(n: usize) -> Vec<Body> {
    (0..n).map(|_|
               Body {
                   x: Vec2(random(), random()),
                   v: Vec2(0.0, 0.0),
                   a: Vec2::zero(),
                   m: 1.0,
               }
    ).collect()
}
