pub enum Curve {
    /// Always has the same value.
    Constant(f64),
    /// linearly go from a to a+b (the second value is how much to move, NOT the destination!)
    Linear(f64, f64),
    /// Chains multiple Curves together. Obviously, the curve's values should be the same at the points where they meet, but this is not strictly necessary. The f64 values in the tuple are the length for the corresponding curve. If their sum is less than 1, the end will use the value the final curve returned for 1.
    Chain(Vec<(Self, f64)>),
    /// Allows a custom function to determine the value.
    Custom(Box<fn(f64) -> f64>),
}
impl Curve {
    /// For a range from 0 to 1, returns a value (mostly also from 0 to 1, but could exceed the two bounds).
    pub fn get_value(&self, progress: f64) -> f64 {
        match self {
            Curve::Constant(v) => v.clone(),
            Curve::Linear(start, dif) => start + dif * progress,
            Curve::Chain(chain) => {
                let mut out_progress = 0.0;
                let mut total_progress = 0.0;
                for curve in chain {
                    if total_progress + curve.1 > progress {
                        out_progress = curve.0.get_value((progress - total_progress).min(1.0) / curve.1);
                        break;
                    };
                    total_progress += curve.1;
                };
                out_progress
            },
            Curve::Custom(f) => f(progress),
        }
    }
    fn get_attributes(&self) -> Vec<(String, CurveAttribute)> { Vec::new() }
}

pub enum CurveAttribute {
    flag(bool),
    float(f64),
    int(i32),
    list(Vec<CurveAttribute>),
}
impl CurveAttribute {
    pub fn to_string(&self) -> String {
        let x = |f: f64| f*f;
        match self {
            CurveAttribute::flag(v) => if *v { "☑" } else { "☐" }.to_string(),
            CurveAttribute::float(v) => v.to_string(),
            CurveAttribute::int(v) => v.to_string(),
            CurveAttribute::list(v) => "List".to_string(),
        }
    }
}