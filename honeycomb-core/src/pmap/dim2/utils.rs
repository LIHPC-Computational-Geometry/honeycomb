//! [`PMap2`] utilities implementations
//!
//! <div class="warning">
//!
//! **This code is only compiled if the `utils` feature is enabled.**
//!
//! </div>
//!
//! This module contains utility code for the [`PMap2`] structure that is gated behind the `utils`
//! feature.

// ------ IMPORTS

use super::PMAP2_BETA;
use crate::geometry::CoordsFloat;
use crate::pmap::dim2::structure::PMap2;
use crate::prelude::DartIdentifier;
use std::sync::atomic::Ordering;

// ------ CONTENT

/// **Utilities**
impl<T: CoordsFloat> PMap2<T> {
    /// Set the value of the specified beta function of a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `val: DartIdentifier` -- Value of the image of `dart_id` through the beta `I` function.
    ///
    /// ## Generic
    ///
    /// - `const I: u8` -- Beta function to edit.
    ///
    pub fn set_beta<const I: u8>(&self, dart_id: DartIdentifier, val: DartIdentifier) {
        self.betas[dart_id as usize][I as usize].store(val, Ordering::Relaxed);
    }

    /// Set the values of the beta functions of a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `[b0, b1, b2]: [DartIdentifier; 3]` -- Value of the images as
    ///   *[β<sub>0</sub>(dart), β<sub>1</sub>(dart), β<sub>2</sub>(dart)]*
    ///
    pub fn set_betas(&self, dart_id: DartIdentifier, [b0, b1, b2]: [DartIdentifier; PMAP2_BETA]) {
        // using individual stores to keep a non-mutable sig
        self.betas[dart_id as usize][0].store(b0, Ordering::Relaxed);
        self.betas[dart_id as usize][1].store(b1, Ordering::Relaxed);
        self.betas[dart_id as usize][2].store(b2, Ordering::Relaxed);
    }
}
