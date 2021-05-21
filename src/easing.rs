#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub trait Easing: Copy + Clone + std::fmt::Debug {
    fn new_with_value(value: f64) -> Self;
    fn get(&self) -> f64;
    fn step(&mut self, dt: f64);
    fn set_goal(&mut self, goal: f64);
    fn get_goal(&self) -> f64;
}

#[derive(Copy, Clone, Debug)]
pub struct DumbExpEasing {
    pub gamma: f64,

    value: f64,
    goal: f64,
}

impl DumbExpEasing {
    pub fn new(value: f64, gamma: f64, goal: f64) -> DumbExpEasing {
        Self { value, gamma, goal }
    }
}

impl Easing for DumbExpEasing {
    fn new_with_value(value: f64) -> Self {
        DumbExpEasing::new(value, 4.0, value)
    }

    fn get(&self) -> f64 {
        self.value
    }

    fn step(&mut self, dt: f64) {
        self.value -= dt * self.gamma * (self.value - self.goal);
    }

    fn set_goal(&mut self, goal: f64) {
        self.goal = goal;
    }

    fn get_goal(&self) -> f64 {
        self.goal
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SinEasing {
    t: f64,

    a: f64,
    b: f64,
    omega: f64,
    phi: f64,

    t1: f64,
}

impl SinEasing {
    fn new(value: f64, t1: f64) -> Self {
        SinEasing {
            t: 0.,
            a: 0.,
            b: value,
            omega: 0.,
            phi: 0.,
            t1: t1,
        }
    }
    fn is_saturated(&self) -> bool {
        self.omega * self.t + self.phi > 0.
    }
    fn get_deriv(&self) -> f64 {
        if self.is_saturated() {
            0.
        } else {
            -self.a * self.omega * (self.omega * self.t + self.phi).sin()
        }
    }
    fn solve_nonlin(mut gamma: f64) -> f64 {
        if gamma <= -1.9 {
            error!("Capping gamma! {} -> -1.9", gamma);
            gamma = -1.9;
        }
        if gamma > 100. {
            error!("Capping gamma! {} -> 100", gamma);
            gamma = 100.;
        }
        use std::f64::consts::PI;

        let mut t = 2.*PI - PI*PI*PI/(2. * gamma + PI*PI);

        for _ in 0..10 {
            let f = t * t.sin() / (t.cos()-1.) - gamma;
            let dfdt = (t * t.sin() * t.sin() + t * (t.cos() - 1.) * t.cos() + t.sin() * (t.cos() - 1.)) / ((t.cos() - 1.) * (t.cos() - 1.));
            t = t - f / dfdt;
        }

        let delta = t * t.sin() / (t.cos()-1.) - gamma;
        if delta.abs() > 0.1 {
            error!("Large delta for SinEasing! Δ = {:?} for gamma={:?}", delta, gamma);
        }

        t
    }
}

impl Easing for SinEasing {
    fn new_with_value(value: f64) -> Self {
        Self::new(value, 0.5)
    }
    fn get(&self) -> f64 {
        if self.is_saturated() {
            self.a + self.b
        } else {
            self.a * (self.omega * self.t + self.phi).cos() + self.b
        }
    }
    fn step(&mut self, dt: f64) {
        self.t += dt;
    }

    fn set_goal(&mut self, goal: f64) {
        let n = goal;
        let m = self.get();
        let k = self.get_deriv();

        let gamma = k * self.t1 / (m - n);
        self.omega = SinEasing::solve_nonlin(gamma) / self.t1;
        self.a = (m - n) / ((self.omega * self.t1).cos() - 1.);
        self.b = n - self.a;
        self.phi = -self.omega * self.t1;

        self.t = 0.;
    }

    fn get_goal(&self) -> f64 {
        self.a + self.b
    }
}

