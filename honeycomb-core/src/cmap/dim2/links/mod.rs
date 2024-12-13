mod one;
mod two;

use stm::{StmResult, Transaction};

use crate::{
    cmap::{CMap2, DartIdType},
    prelude::CoordsFloat,
};

/// # **Link operations**
impl<T: CoordsFloat> CMap2<T> {
    /// # `I`-link operator.
    ///
    /// This operation corresponds to coherently linking two darts via their *β* images. Unlike
    /// sewing, this does not affect associated attributes.
    ///
    /// For a thorough explanation of this operation, its hypothesis & consequences, refer
    /// to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/definitions/sew.html
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Sew dimension.
    /// - `trans: &mut Transaction` -- Transaction associated to the operation.
    /// - `ld: DartIdType` -- First dart ID.
    /// - `rd: DartIdType` -- Second dart ID.
    ///
    /// # Errors
    ///
    /// This method should be called in a transactional context. The `Result` is then used to
    /// validate the transaction; It should not be processed manually. The policy in case of
    /// failure can be defined when creating the transaction, using `Transaction::with_control`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - `I >= 3` or `I == 0`,
    /// - the two darts are not I-linkable.
    pub fn link<const I: u8>(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> StmResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.one_link(trans, ld, rd),
            2 => self.two_link(trans, ld, rd),
            _ => unreachable!(),
        }
    }

    /// # `I`-unlink operator.
    ///
    /// This operation corresponds to unlinking two darts by resetting their *β* images. Unlike
    /// linking, this does not affect associated attributes.
    ///
    /// For a thorough explanation of this operation, its hypothesis & consequences, refer
    /// to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/definitions/sew.html
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Unsew dimension.
    /// - `trans: &mut Transaction` -- Transaction associated to the operation.
    /// - `ld: DartIdType` -- First dart ID.
    ///
    /// The second dart ID is fetched using `I` and `ld`.
    ///
    /// # Errors
    ///
    /// This method should be called in a transactional context. The `Result` is then used to
    /// validate the transaction; It should not be processed manually. The policy in case of
    /// failure can be defined when creating the transaction, using `Transaction::with_control`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - `I >= 3` or `I == 0`
    /// - `ld` is already `I`-free,
    pub fn unlink<const I: u8>(&self, trans: &mut Transaction, ld: DartIdType) -> StmResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.one_unlink(trans, ld),
            2 => self.two_unlink(trans, ld),
            _ => unreachable!(),
        }
    }

    /// # `I`-link operator.
    ///
    /// This variant is equivalent to `sew`, but internally uses a transaction that will
    /// be retried until validated.
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Sew dimension.
    /// - `ld: DartIdType` -- First dart ID.
    /// - `rd: DartIdType` -- Second dart ID.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - `I >= 3` or `I == 0`,
    /// - the two darts are not I-linkable.
    pub fn force_link<const I: u8>(&self, ld: DartIdType, rd: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_link(ld, rd),
            2 => self.force_two_link(ld, rd),
            _ => unreachable!(),
        }
    }

    /// # `I`-unlink operator.
    ///
    /// This variant is equivalent to `sew`, but internally uses a transaction that will
    /// be retried until validated.
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Unsew dimension.
    /// - `ld: DartIdType` -- First dart ID.
    ///
    /// The second dart ID is fetched using `I` and `ld`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - `I >= 3` or `I == 0`
    /// - `ld` is already `I`-free,
    pub fn force_unlink<const I: u8>(&self, ld: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 3);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_unlink(ld),
            2 => self.force_two_unlink(ld),
            _ => unreachable!(),
        }
    }
}
