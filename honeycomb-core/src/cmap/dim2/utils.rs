//! [`CMap2`] utilities implementations

use crate::cmap::{CMap2, DartIdType};
use crate::geometry::CoordsFloat;

use super::CMAP2_BETA;

/// **Utilities**
impl<T: CoordsFloat> CMap2<T> {
    /// Set the value of β<sub>`I`</sub>(`dart_id`) to `new_val`.
    pub fn set_beta<const I: u8>(&self, dart_id: DartIdType, new_val: DartIdType) {
        self.betas[(I, dart_id)].write_atomic(new_val);
    }

    /// Set the values of the beta functions of a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `betas: [DartIdentifier; 3]` -- Value of the images as
    ///   [ β<sub>`0`</sub>(`dart_id`), β<sub>`1`</sub>(`dart_id`), β<sub>`2`</sub>(`dart_id`) ]
    pub fn set_betas(&self, dart_id: DartIdType, [b0, b1, b2]: [DartIdType; CMAP2_BETA]) {
        // store separately to use non-mutable methods
        self.betas[(0, dart_id)].write_atomic(b0);
        self.betas[(1, dart_id)].write_atomic(b1);
        self.betas[(2, dart_id)].write_atomic(b2);
    }
}
