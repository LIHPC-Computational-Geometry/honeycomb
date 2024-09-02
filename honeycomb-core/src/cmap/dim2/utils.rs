//! [`CMap2`] utilities implementations
//!
//! <div class="warning">
//!
//! **This code is only compiled if the `utils` feature is enabled.**
//!
//! </div>
//!
//! This module contains utility code for the [`CMap2`] structure that is gated behind the `utils`
//! feature.

// ------ IMPORTS

use super::CMAP2_BETA;
use crate::geometry::CoordsFloat;
use crate::prelude::{CMap2, DartIdentifier};
use std::{fs::File, io::Write};

// ------ CONTENT

// --- fast overwrite method
impl<T: CoordsFloat> CMap2<T> {
    /// Set the values of the beta functions of a dart.
    ///
    /// <div class="warning">
    ///
    /// **This method is only compiled if the `utils` feature is enabled.**
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `dart_id: DartIdentifier` -- ID of the dart of interest.
    /// - `betas: [DartIdentifier; 3]` -- Value of the images as
    ///   *[β<sub>0</sub>(dart), β<sub>1</sub>(dart), β<sub>2</sub>(dart)]*
    ///
    pub fn set_betas(&mut self, dart_id: DartIdentifier, betas: [DartIdentifier; CMAP2_BETA]) {
        self.betas[dart_id as usize] = betas;
    }
}

// --- size methods
impl<T: CoordsFloat> CMap2<T> {
    /// Computes the total allocated space dedicated to the map.
    ///
    /// <div class="warning">
    ///
    /// **This method is only compiled if the `utils` feature is enabled.**
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `rootname: &str` -- root of the filename used to save results.
    ///
    /// # Return / Panic
    ///
    /// The results of this method is saved as a csv file named `<rootname>_allocated.csv`.
    /// The csv file is structured as follows:
    ///
    /// ```text
    /// key, memory (bytes)
    /// cat1_member1, val
    /// cat1_member2, val
    /// cat1_total, val
    /// cat2_member1, val
    /// cat2_member2, val
    /// cat2_member3, val
    /// cat2_total, val
    /// cat3_member1, val
    /// cat3_total, val
    /// ```
    ///
    /// It is mainly designed to be used in dedicated scripts for plotting & analysis.
    ///
    /// The metod may panic if, for any reason, it is unable to write to the file.
    ///
    /// # Example
    ///
    /// An example going over all three `size` methods is provided in the `honeycomb-utils`
    /// crate. You can run it using the following command:
    ///
    /// ```shell
    /// cargo run --example memory_usage
    /// ```
    ///
    /// The output data can be visualized using the `memory_usage.py` script.
    ///
    /// # Panics
    ///
    /// The method may panic if, at any point, the program cannot write into the output file.
    ///
    pub fn allocated_size(&self, rootname: &str) {
        let mut file = File::create(rootname.to_owned() + "_allocated.csv").unwrap();
        writeln!(file, "key, memory (bytes)").unwrap();

        // beta
        let mut beta_total = 0;
        (0..3).for_each(|beta_id| {
            let mem = self.betas.capacity() * std::mem::size_of::<DartIdentifier>();
            writeln!(file, "beta_{beta_id}, {mem}").unwrap();
            beta_total += mem;
        });
        writeln!(file, "beta_total, {beta_total}").unwrap();

        // cells
        // using 2 * sizeof(f64) bc sizeof(array) always is the size of a pointer
        let geometry_vertex = self.vertices.allocated_size();
        let geometry_total = geometry_vertex;
        writeln!(file, "geometry_vertex, {geometry_vertex}").unwrap();
        writeln!(file, "geometry_total, {geometry_total}").unwrap();

        // others
        let others_freedarts = self.unused_darts.len();
        let others_counters = 2 * std::mem::size_of::<usize>();
        let others_total = others_freedarts + others_counters;
        writeln!(file, "others_freedarts, {others_freedarts}").unwrap();
        writeln!(file, "others_counters, {others_counters}").unwrap();
        writeln!(file, "others_total, {others_total}").unwrap();
    }

    /// Computes the total used space dedicated to the map.
    ///
    /// <div class="warning">
    ///
    /// **This method is only compiled if the `utils` feature is enabled.**
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `rootname: &str` -- root of the filename used to save results.
    ///
    /// # Return / Panic
    ///
    /// The results of this method is saved as a csv file named `<rootname>_allocated.csv`.
    /// The csv file is structured as follows:
    ///
    /// ```text
    /// key, memory (bytes)
    /// cat1_member1, val
    /// cat1_member2, val
    /// cat1_total, val
    /// cat2_member1, val
    /// cat2_member2, val
    /// cat2_member3, val
    /// cat2_total, val
    /// cat3_member1, val
    /// cat3_total, val
    /// ```
    ///
    /// It is mainly designed to be used in dedicated scripts for plotting & analysis.
    ///
    /// The metod may panic if, for any reason, it is unable to write to the file.
    ///
    /// # Example
    ///
    /// An example going over all three `size` methods is provided in the `honeycomb-utils`
    /// crate. You can run it using the following command:
    ///
    /// ```shell
    /// cargo run --example memory_usage
    /// ```
    ///
    /// The output data can be visualized using the `memory_usage.py` script.
    ///
    /// # Panics
    ///
    /// The method may panic if, at any point, the program cannot write into the output file.
    ///
    pub fn effective_size(&self, rootname: &str) {
        let mut file = File::create(rootname.to_owned() + "_effective.csv").unwrap();
        writeln!(file, "key, memory (bytes)").unwrap();

        // beta
        let mut beta_total = 0;
        (0..3).for_each(|beta_id| {
            let mem = self.n_darts * std::mem::size_of::<DartIdentifier>();
            writeln!(file, "beta_{beta_id}, {mem}").unwrap();
            beta_total += mem;
        });
        writeln!(file, "beta_total, {beta_total}").unwrap();

        // cells
        // using 2 * sizeof(f64) bc sizeof(array) always is the size of a pointer
        let geometry_vertex = self.vertices.effective_size();
        let geometry_total = geometry_vertex;
        writeln!(file, "geometry_vertex, {geometry_vertex}").unwrap();
        writeln!(file, "geometry_total, {geometry_total}").unwrap();

        // others
        let others_freedarts = self.unused_darts.len();
        let others_counters = 2 * std::mem::size_of::<usize>();
        let others_total = others_freedarts + others_counters;
        writeln!(file, "others_freedarts, {others_freedarts}").unwrap();
        writeln!(file, "others_counters, {others_counters}").unwrap();
        writeln!(file, "others_total, {others_total}").unwrap();
    }

    /// Computes the actual used space dedicated to the map.
    ///
    /// *Actual used space* refers to the total used space minus empty spots
    /// in the structure.
    ///
    /// <div class="warning">
    ///
    /// **This method is only compiled if the `utils` feature is enabled.**
    ///
    /// </div>
    ///
    /// # Arguments
    ///
    /// - `rootname: &str` -- root of the filename used to save results.
    ///
    /// # Return / Panic
    ///
    /// The results of this method is saved as a csv file named `<rootname>_allocated.csv`.
    /// The csv file is structured as follows:
    ///
    /// ```text
    /// key, memory (bytes)
    /// cat1_member1, val
    /// cat1_member2, val
    /// cat1_total, val
    /// cat2_member1, val
    /// cat2_member2, val
    /// cat2_member3, val
    /// cat2_total, val
    /// cat3_member1, val
    /// cat3_total, val
    /// ```
    ///
    /// It is mainly designed to be used in dedicated scripts for plotting & analysis.
    ///
    /// The metod may panic if, for any reason, it is unable to write to the file.
    ///
    /// # Example
    ///
    /// An example going over all three `size` methods is provided in the `honeycomb-utils`
    /// crate. You can run it using the following command:
    ///
    /// ```shell
    /// cargo run --example memory_usage
    /// ```
    ///
    /// The output data can be visualized using the `memory_usage.py` script.
    ///
    /// # Panics
    ///
    /// The method may panic if, at any point, the program cannot write into the output file.
    ///
    pub fn used_size(&self, rootname: &str) {
        let mut file = File::create(rootname.to_owned() + "_used.csv").unwrap();
        writeln!(file, "key, memory (bytes)").unwrap();

        let n_used_darts = self.n_darts - self.unused_darts.len();

        // beta
        let mut beta_total = 0;
        (0..3).for_each(|beta_id| {
            let mem = n_used_darts * std::mem::size_of::<DartIdentifier>();
            writeln!(file, "beta_{beta_id}, {mem}").unwrap();
            beta_total += mem;
        });
        writeln!(file, "beta_total, {beta_total}").unwrap();

        // cells
        // using 2 * sizeof(f64) bc sizeof(array) always is the size of a pointer
        let geometry_vertex = self.vertices.used_size();
        let geometry_total = geometry_vertex;
        writeln!(file, "geometry_vertex, {geometry_vertex}").unwrap();
        writeln!(file, "geometry_total, {geometry_total}").unwrap();

        // others
        let others_freedarts = self.unused_darts.len();
        let others_counters = 2 * std::mem::size_of::<usize>();
        let others_total = others_freedarts + others_counters;
        writeln!(file, "others_freedarts, {others_freedarts}").unwrap();
        writeln!(file, "others_counters, {others_counters}").unwrap();
        writeln!(file, "others_total, {others_total}").unwrap();
    }
}
