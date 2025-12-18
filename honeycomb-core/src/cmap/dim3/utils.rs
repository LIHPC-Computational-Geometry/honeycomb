//! [`CMap3`] utilities implementations
//!
//! This module contains utility code for the [`CMap3`] structure.

use rayon::prelude::*;

use crate::cmap::{CMap3, DartIdType, OrbitPolicy};
use crate::geometry::CoordsFloat;
use crate::stm::atomically;

use super::CMAP3_BETA;

/// **Utilities**
impl<T: CoordsFloat> CMap3<T> {
    /// Set the value of the specified beta function of a dart.
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Beta function to edit.
    /// - `dart_id: DartIdType` -- ID of the dart of interest.
    /// - `val: DartIdType` -- New value of *β<sub>`I`</sub>(`dart_id`)*.
    pub fn set_beta<const I: u8>(&self, dart_id: DartIdType, val: DartIdType) {
        atomically(|t| self.betas[(I, dart_id)].write(t, val));
    }

    /// Set the values of the beta functions of a dart.
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdType` -- ID of the dart of interest.
    /// - `betas: [DartIdType; 4]` -- New values of
    ///   *[β<sub>0</sub>(dart), β<sub>1</sub>(dart), β<sub>2</sub>(dart), β<sub>3</sub>(dart)]*
    ///
    pub fn set_betas(&self, dart_id: DartIdType, [b0, b1, b2, b3]: [DartIdType; CMAP3_BETA]) {
        // store separately to use non-mutable methods
        atomically(|t| {
            self.betas[(0, dart_id)].write(t, b0)?;
            self.betas[(1, dart_id)].write(t, b1)?;
            self.betas[(2, dart_id)].write(t, b2)?;
            self.betas[(3, dart_id)].write(t, b3)?;
            Ok(())
        });
    }

    pub fn update_vertex_id_cache(&self) {
        if let Some(ref vids) = self.vertex_ids {
            (1..self.n_darts() as DartIdType)
                .into_par_iter()
                .for_each(|d| {
                    let min = self
                        .orbit(OrbitPolicy::Vertex, d)
                        .min()
                        .expect("E: unreachable");
                    atomically(|t| vids[d as usize].write(t, min));
                });
        }
    }
}
