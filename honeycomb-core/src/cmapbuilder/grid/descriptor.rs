//! Main grid descriptor implementation

// ------ IMPORTS

#[allow(deprecated)]
use crate::utils::GridBuilder;
use crate::{BuilderError, CoordsFloat};

// ------ CONTENT

#[allow(deprecated)]
/// Temporary type alias before [`GridBuilder`] is renamed to this.
pub type GridDescriptor<T> = GridBuilder<T>;

// --- parsing routine

macro_rules! check_parameters {
    ($id: ident, $msg: expr) => {
        if $id.is_sign_negative() | $id.is_zero() {
            return Err(BuilderError::InvalidParameters($msg));
        }
    };
}

#[allow(deprecated)]
impl<T: CoordsFloat> GridDescriptor<T> {
    /// Parse provided grid parameters to provide what's used to actually generate the grid.
    pub(crate) fn parse(self) -> Result<([usize; 2], [T; 2]), BuilderError> {
        match (self.n_cells, self.len_per_cell, self.lens) {
            // from # cells and lengths per cell
            (Some([nx, ny, _]), Some([lpx, lpy, _]), lens) => {
                if lens.is_some() {
                    println!("W: All three grid parameters were specified, total lengths will be ignored");
                }
                #[rustfmt::skip]
                check_parameters!(lpx, "Specified length per x cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "Specified length per y cell is either null or negative");
                Ok(([nx, ny], [lpx, lpy]))
            }
            // from # cells and total lengths
            (Some([nx, ny, _]), None, Some([lx, ly, _])) => {
                #[rustfmt::skip]
                check_parameters!(lx, "Specified grid length along x is either null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "Specified grid length along y is either null or negative");
                Ok((
                    [nx, ny],
                    [lx / T::from(nx).unwrap(), ly / T::from(ny).unwrap()],
                ))
            }
            // from lengths per cell and total lengths
            (None, Some([lpx, lpy, _]), Some([lx, ly, _])) => {
                #[rustfmt::skip]
                check_parameters!(lpx, "Specified length per x cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "Specified length per y cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lx, "Specified grid length along x is either null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "Specified grid length along y is either null or negative");
                Ok((
                    [
                        (lx / lpx).ceil().to_usize().unwrap(),
                        (ly / lpy).ceil().to_usize().unwrap(),
                    ],
                    [lpx, lpy],
                ))
            }
            (_, _, _) => Err(BuilderError::MissingParameters(
                "GridBuilder: insufficient building parameters",
            )),
        }
    }
}
