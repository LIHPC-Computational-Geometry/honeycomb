//! Generic attributes implementation
//!
//!

// ------ IMPORTS

// ------ CONTENT

pub trait AttributeLogic: Sized {
    fn merge(lhs: Self, rhs: Self) -> Self;

    fn split(lhs: Self) -> (Self, Self);

    fn merge_undefined(lhs: Option<Self>) -> Self {
        lhs.unwrap() // todo: choose a policy for default behavior
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
