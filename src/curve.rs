pub enum Curve {
    /// Always has the same value.
    Constant(f64),
    /// linearly go from a to b
    Linear(BCurve, BCurve),
    /// Smoothly go from one point to another, where f(0.0) = self.0, f(1.0) = self.1, and f'(0.0) = f'(1.0) = 0.0 (flattened out at both ends)
    SmoothFlat(BCurve, BCurve),
    /// Chains multiple Curves together. Obviously, the curve's values should be the same at the points where they meet, but this is not strictly necessary. The f64 values in the tuple are the length for the corresponding curve. If their sum is less than 1, the end will use the value the final curve returned for 1.
    Chain(Vec<(Curve, f64)>),
    Program(std::path::PathBuf),
    // ProgramPersistent((), ), // TODO!
}
impl Clone for Curve { fn clone(&self) -> Self { match self {
    Curve::Constant(a) => Curve::Constant(a.clone()),
    Curve::Linear(a, b) => Curve::Linear(a.clone(), b.clone()),
    Curve::SmoothFlat(a, b) => Curve::SmoothFlat(a.r().clone().b(), b.r().clone().b()),
    Curve::Chain(a) => Curve::Chain({
        let mut nvec = Vec::with_capacity(a.len());
        for b in a {
            nvec.push((b.0.clone(), b.1.clone()));
        };
        nvec
    }),
    Curve::Program(p) => Curve::Program(p.clone()),
} } }

pub struct BCurve {
    pub c: Box<Curve>,
} impl BCurve {
    pub fn n(c: Curve) -> Self { Self { c: Box::new(c), } }
    pub fn c(self) -> Curve { *self.c }
    pub fn r(&self) -> &Curve { self.c.as_ref() }
} impl Clone for BCurve {
    fn clone(&self) -> Self { self.c.clone().b() }
}
impl Curve {
    pub fn b(self) -> BCurve { BCurve::n(self) }
    /// For a range from 0 to 1, returns a value (mostly also from 0 to 1, but could exceed the two bounds).
    pub fn get_value(&self, progress: f64) -> f64 {
        match self {
            Self::Constant(v) => *v,
            Self::Linear(start, end) => {
                let start = start.c.get_value(progress);
                let dif = end.c.get_value(progress) - start;
                start + dif * progress
            },
            Self::Chain(chain) => {
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
            Self::SmoothFlat(x1, x2) => {
                let x1 = x1.c.get_value(progress);
                let x2 = x2.c.get_value(progress);
                let factor = -2.0 * progress * progress * progress + 3.0 * progress * progress;
                x1 + (x2 - x1) * factor
            },
            Self::Program(p) => {
                let txt = String::from_utf8(
                    std::process::Command::new(p)
                        .arg(format!("{}", progress).as_str())
                        .output().unwrap().stdout
                ).expect("Program output was not valid UTF-8");
                let txt = txt.split('\n').next().unwrap();
                txt.parse().expect(format!("Program output could not be parsed into a float: '{}'", txt).as_str())
            },
        }
    }
}