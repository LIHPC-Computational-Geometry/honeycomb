//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

// ------ IMPORTS

// ------ CONTENT

cfg_if::cfg_if! {
    if #[cfg(feature = "single_precision")] {
        pub type FloatType = f32;
    } else {
        pub type FloatType = f64;
    }
}

pub struct Coords2 {
    pub x: FloatType,
    pub y: FloatType,
}

impl From<(FloatType, FloatType)> for Coords2 {
    fn from((x, y): (FloatType, FloatType)) -> Self {
        Self { x, y }
    }
}

impl From<[FloatType; 2]> for Coords2 {
    fn from([x, y]: [FloatType; 2]) -> Self {
        Self { x, y }
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn some_test() {
        assert_eq!(1, 1);
    }
}
