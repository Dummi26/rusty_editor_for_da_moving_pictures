use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::project::{SharedCurves, SharedCurvesId};

// NOTE: Cloning might seem a little weird because Shared and Owned types might be mixed.

#[derive(Clone)]
pub enum Curve {
    Owned(Box<CurveData>),
    Shared(SharedCurvesId, Box<CurveData>),
}
impl Curve {
    pub fn is_owned(&self) -> bool {
        if let Self::Owned(_) = self {
            true
        } else {
            false
        }
    }
    /// For a range from 0 to 1, returns a value (mostly also from 0 to 1, but could exceed the two bounds).
    pub fn get_value(&self, progress: f64) -> f64 {
        match self {
            Self::Owned(v) | Self::Shared(_, v) => v.get_value(progress),
        }
    }
    pub fn update(&mut self, shared_curves: &SharedCurves) {
        if let Self::Shared(id, data) = self {
            *data = Box::new(shared_curves.get(&id).unwrap().clone());
        }
    }
    pub fn to_shared(&mut self, shared_curves: &SharedCurves) {
        if let Self::Owned(curve) = self {
            *self = Self::Shared(
                shared_curves.insert(*curve.clone()),
                Box::new(*curve.clone()),
            )
        }
    }
}

impl From<CurveData> for Curve {
    fn from(value: CurveData) -> Self {
        Self::Owned(Box::new(value))
    }
}

pub enum CurveData {
    /// Always has the same value.
    Constant(f64),
    /// linearly go from a to b
    Linear(Curve, Curve),
    /// Smoothly go from one point to another, where f(0.0) = self.0, f(1.0) = self.1, and f'(0.0) = f'(1.0) = 0.0 (flattened out at both ends)
    SmoothFlat(Curve, Curve),
    /// Chains multiple Curves together. Obviously, the curve's values should be the same at the points where they meet, but this is not strictly necessary. The f64 values in the tuple are the length for the corresponding curve. If their sum is less than 1, the end will use the value the final curve returned for 1.
    Chain(Vec<(Curve, f64)>),
    Program(
        crate::external_program::ExternalProgram,
        CurveExternalProgramMode,
    ),
    // ProgramPersistent((), ), // TODO!
}
impl Clone for CurveData {
    fn clone(&self) -> Self {
        match self {
            Self::Constant(a) => Self::Constant(a.clone()),
            Self::Linear(a, b) => Self::Linear(a.clone(), b.clone()),
            Self::SmoothFlat(a, b) => Self::SmoothFlat(a.clone(), b.clone()),
            Self::Chain(a) => Self::Chain({
                let mut nvec = Vec::with_capacity(a.len());
                for b in a {
                    nvec.push((b.0.clone(), b.1.clone()));
                }
                nvec
            }),
            Self::Program(p, m) => Self::Program(p.clone(), *m),
        }
    }
}

impl CurveData {
    /// For a range from 0 to 1, returns a value (mostly also from 0 to 1, but could exceed the two bounds).
    pub fn get_value(&self, progress: f64) -> f64 {
        match self {
            Self::Constant(v) => *v,
            Self::Linear(start, end) => {
                let start = start.get_value(progress);
                let dif = end.get_value(progress) - start;
                start + dif * progress
            }
            Self::Chain(chain) => 'get_from_chain: {
                for i in 0..chain.len() {
                    let this = &chain[i];
                    let (curve, start) = this;
                    let end = match chain.get(i + 1) {
                        Some(v) => v.1,
                        None => 1.0,
                    };
                    if end > progress {
                        break 'get_from_chain curve.get_value((progress - start) / (end - start));
                    }
                }
                match chain.last() {
                    Some(last) => last.0.get_value(1.0),
                    None => 1.0,
                }
            }
            Self::SmoothFlat(x1, x2) => {
                let x1 = x1.get_value(progress);
                let x2 = x2.get_value(progress);
                let factor = -2.0 * progress * progress * progress + 3.0 * progress * progress;
                x1 + (x2 - x1) * factor
            }
            Self::Program(p, m) => {
                let txt =
                    String::from_utf8(p.get_next(format!("{}", progress).as_bytes()).unwrap())
                        .expect("Program output was not valid UTF-8");
                let txt = txt.split('\n').next().unwrap();
                txt.parse().expect(
                    format!("Program output could not be parsed into a float: '{}'", txt).as_str(),
                )
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum CurveExternalProgramMode {
    String,
}
