use crate::{
    cmap::{CMap3, DartIdType},
    prelude::CoordsFloat,
};

/// **Serialization methods**
impl<T: CoordsFloat> CMap3<T> {
    // --- Custom

    /// Serialize the map under a custom format.
    ///
    /// The format specification is described in the [user guide]().
    pub fn serialize(&self, mut writer: impl std::fmt::Write) {
        let n_darts = self.n_darts();

        writeln!(writer, "[META]").expect("E: couldn't write to file");
        writeln!(
            writer,
            "{} {} {}",
            env!("CARGO_PKG_VERSION"), // indicates which version was used to generate the file
            3,
            n_darts - 1
        )
        .expect("E: couldn't write to file");
        writeln!(writer).expect("E: couldn't write to file"); // not required, but nice

        writeln!(writer, "[BETAS]").expect("E: couldn't write to file");
        let width = n_darts.to_string().len();
        let mut b0 = String::with_capacity(self.n_darts() * 2);
        let mut b1 = String::with_capacity(self.n_darts() * 2);
        let mut b2 = String::with_capacity(self.n_darts() * 2);
        let mut b3 = String::with_capacity(self.n_darts() * 2);
        std::thread::scope(|s| {
            s.spawn(|| {
                // convoluted bc this prevents ephemeral allocs
                use std::fmt::Write;
                let mut buf = String::new();
                (0..n_darts as DartIdType).for_each(|d| {
                    write!(&mut buf, "{:>width$} ", self.beta::<0>(d)).unwrap();
                    b0.push_str(buf.as_str());
                    buf.clear();
                });
            });
            s.spawn(|| {
                // convoluted bc this prevents ephemeral allocs
                use std::fmt::Write;
                let mut buf = String::new();
                (0..n_darts as DartIdType).for_each(|d| {
                    write!(&mut buf, "{:>width$} ", self.beta::<1>(d)).unwrap();
                    b1.push_str(buf.as_str());
                    buf.clear();
                });
            });
            s.spawn(|| {
                // convoluted bc this prevents ephemeral allocs
                use std::fmt::Write;
                let mut buf = String::new();
                (0..n_darts as DartIdType).for_each(|d| {
                    write!(&mut buf, "{:>width$} ", self.beta::<2>(d)).unwrap();
                    b2.push_str(buf.as_str());
                    buf.clear();
                });
            });
            s.spawn(|| {
                // convoluted bc this prevents ephemeral allocs
                use std::fmt::Write;
                let mut buf = String::new();
                (0..n_darts as DartIdType).for_each(|d| {
                    write!(&mut buf, "{:>width$} ", self.beta::<3>(d)).unwrap();
                    b3.push_str(buf.as_str());
                    buf.clear();
                });
            });
        });
        writeln!(writer, "{}", b0.trim()).expect("E: couldn't write to file");
        writeln!(writer, "{}", b1.trim()).expect("E: couldn't write to file");
        writeln!(writer, "{}", b2.trim()).expect("E: couldn't write to file");
        writeln!(writer, "{}", b3.trim()).expect("E: couldn't write to file");
        writeln!(writer).expect("E: couldn't write to file"); // not required, but nice

        writeln!(writer, "[UNUSED]").expect("E: couldn't write to file");
        self.unused_darts
            .iter()
            .enumerate()
            .filter(|(_, v)| v.read_atomic())
            .for_each(|(i, _)| {
                write!(writer, "{i} ").unwrap();
            });
        writeln!(writer).expect("E: couldn't write to file"); // required
        writeln!(writer).expect("E: couldn't write to file"); // not required, but nice

        writeln!(writer, "[VERTICES]").expect("E: couldn't write to file");
        self.iter_vertices().for_each(|v| {
            if let Some(val) = self.force_read_vertex(v) {
                writeln!(
                    writer,
                    "{v} {} {} {}",
                    val.0.to_f64().unwrap(),
                    val.1.to_f64().unwrap(),
                    val.2.to_f64().unwrap(),
                )
                .expect("E: couldn't write to file");
            }
        });
    }
}
