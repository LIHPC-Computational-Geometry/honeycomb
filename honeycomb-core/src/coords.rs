//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ MODULE DECLARATIONS

// ------ IMPORTS

// ------ CONTENT

pub type FloatType = cfg_if::cfg_if! {
    if #[cfg(feature = "single_precision")] {
        f32
    } else {
        f64
    }
};

pub struct Coords2 {
    pub x: FloatType,
    pub y: FloatType,
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
