//! [`CMap2`] utilities implementations

// ------ IMPORTS

use super::CMAP2_BETA;
use crate::geometry::CoordsFloat;
use crate::prelude::{CMap2, DartIdType};
use std::{fs::File, io::Write};
use stm::atomically;

// ------ CONTENT

/// **Utilities**
impl<T: CoordsFloat> CMap2<T> {
    /// Set the value of the specified beta function of a dart.
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Beta function to edit.
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `val: DartIdentifier` -- Value of the image of `dart_id` through the beta `I` function.
    ///
    pub fn set_beta<const I: u8>(&self, dart_id: DartIdType, val: DartIdType) {
        atomically(|trans| self.betas[(I, dart_id)].write(trans, val));
    }

    /// Set the values of the beta functions of a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `betas: [DartIdentifier; 3]` -- Value of the images as
    ///   *[β<sub>0</sub>(dart), β<sub>1</sub>(dart), β<sub>2</sub>(dart)]*
    ///
    pub fn set_betas(&self, dart_id: DartIdType, [b0, b1, b2]: [DartIdType; CMAP2_BETA]) {
        // store separately to use non-mutable methods
        atomically(|trans| {
            self.betas[(0, dart_id)].write(trans, b0)?;
            self.betas[(1, dart_id)].write(trans, b1)?;
            self.betas[(2, dart_id)].write(trans, b2)?;
            Ok(())
        });
    }
}
