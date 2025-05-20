/// Zero is no zoom. Negative zoom levels zoom out, positive zoom levels zoom in.
#[derive(Debug, Default, Copy, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct ZoomLevel(pub i8);

impl ZoomLevel {
    pub fn as_f64(&self) -> f64 {
        (self.0 as f64).exp2()
    }
}
impl From<i8> for ZoomLevel {
    fn from(value: i8) -> Self {
        Self(value)
    }
}
