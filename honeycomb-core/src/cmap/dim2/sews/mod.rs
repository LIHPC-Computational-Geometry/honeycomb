mod one;
mod two;

use crate::cmap::{CMap2, DartIdType, SewError};
use crate::geometry::CoordsFloat;
use crate::stm::{Transaction, TransactionClosureResult, atomically_with_err};

/// # **Sew implementations**
impl<T: CoordsFloat> CMap2<T> {
    /// `I`-sew operator.
    ///
    /// # Description
    ///
    /// This operation corresponds to:
    /// - coherently linking two darts via their *β* images,
    /// - merging the attributes associated to their respective original `I`-cells.
    ///
    /// For a thorough explanation of this operation, its hypothesis & consequences, refer
    /// to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/definitions/sew.html
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Sew dimension.
    /// - `t: &mut Transaction` -- Transaction associated to the operation.
    /// - `ld: DartIdType` -- First dart ID.
    /// - `rd: DartIdType` -- Second dart ID.
    ///
    /// # Errors
    ///
    /// This variant will abort the sew operation and raise an error if:
    /// - the transaction cannot be completed,
    /// - one (or more) attribute merge fails,
    /// - for `I == 2`: orientation is inconsistent.
    ///
    /// The returned error can be used in conjunction with transaction control to avoid any
    /// modifications in case of failure at attribute level. The user can then choose to retry or
    /// abort as he wishes using `Transaction::with_control`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - `I >= 3` or `I == 0`,
    /// - the two darts are not `I`-sewable.
    pub fn sew<const I: u8>(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.one_sew(t, ld, rd),
            2 => self.two_sew(t, ld, rd),
            _ => unreachable!(),
        }
    }

    /// `I`-unsew operator.
    ///
    /// # Description
    ///
    /// This operation corresponds to:
    /// - unlinking two darts by resetting their *β* images,
    /// - splitting the attributes associated to the original `I`-cell.
    ///
    /// For a thorough explanation of this operation, its hypothesis & consequences, refer
    /// to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/definitions/sew.html
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Unsew dimension.
    /// - `t: &mut Transaction` -- Transaction associated to the operation.
    /// - `ld: DartIdType` -- First dart ID.
    ///
    /// The second dart ID is fetched using `I` and `ld`.
    ///
    /// # Errors
    ///
    /// This variant will abort the unsew operation and raise an error if:
    /// - the transaction cannot be completed,
    /// - one (or more) attribute split fails,
    ///
    /// The returned error can be used in conjunction with transaction control to avoid any
    /// modifications in case of failure at attribute level. The user can then choose to retry or
    /// abort as he wishes using `Transaction::with_control`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - `I >= 3` or `I == 0`,
    /// - `ld` is already `I`-free.
    pub fn unsew<const I: u8>(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.one_unsew(t, ld),
            2 => self.two_unsew(t, ld),
            _ => unreachable!(),
        }
    }

    #[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
    /// `I`-sew operator.
    ///
    /// This variant is equivalent to [`sew`][Self::sew], but internally uses a transaction that
    /// will be retried until validated.
    pub fn force_sew<const I: u8>(&self, ld: DartIdType, rd: DartIdType) -> Result<(), SewError> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => atomically_with_err(|t| self.one_sew(t, ld, rd)),
            2 => atomically_with_err(|t| self.two_sew(t, ld, rd)),
            _ => unreachable!(),
        }
    }

    #[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
    /// `I`-unsew operator.
    ///
    /// This variant is equivalent to [`unsew`][Self::unsew], but internally uses a transaction that
    /// will be retried until validated.
    pub fn force_unsew<const I: u8>(&self, ld: DartIdType) -> Result<(), SewError> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => atomically_with_err(|t| self.one_unsew(t, ld)),
            2 => atomically_with_err(|t| self.two_unsew(t, ld)),
            _ => unreachable!(),
        }
    }
}
