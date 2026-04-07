
pub struct Interval{
    pub min: f64,
    pub max: f64
}

impl Interval {
    pub fn new(min: f64, max: f64) -> Self{
        Self {
            min, max
        }
    }
    
    pub fn empty() -> Self {
        Self {
            min: 99999999.0,
            max: -99999999.0
        }
    }

    pub fn universe() -> Self {
        Self {
            min: -99999999.0,
            max: 99999999.0
        }
    }
    
    pub fn size(&self) -> f64 {
        self.max - self.min
    }
    
    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }
    
    pub fn surround(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }
    
    pub fn clamp(&self, x: f64) -> f64 {
        x.min(self.max).max(self.min)
    }
}