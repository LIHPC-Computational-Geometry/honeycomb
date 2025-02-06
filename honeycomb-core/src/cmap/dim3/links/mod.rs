mod one;
mod three;
mod two;

use crate::stm::{StmClosureResult, Transaction};

use crate::{
    cmap::{CMap3, DartIdType},
    prelude::CoordsFloat,
};

/// # **Link operations**
impl<T: CoordsFloat> CMap3<T> {
    /// `I`-link operator.
    ///
    /// # Description
    ///
    /// This operation corresponds to coherently linking two darts via their *β* images. Unlike
    /// sewing, this does not alter associated attributes. For a thorough explanation of this
    /// operation, its hypothesis & consequences, refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/definitions/sew.html
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Link dimension.
    /// - `trans: &mut Transaction` -- Transaction associated to the operation.
    /// - `lhs_dart_id: DartIdType` -- First dart ID.
    /// - `rhs_dart_id: DartIdType` -- Second dart ID.
    ///
    /// # Errors
    ///
    /// This method should be called in a transactional context. The `Result` is then used to
    /// validate the transaction; Errors should not be processed manually, only processed via the
    /// `?` operator. The policy in case of failure can be defined when creating the transaction,
    /// using `Transaction::with_control`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - `I >= 4` or `I == 0`,
    /// - the two darts are not `I`-linkable.
    pub fn link<const I: u8>(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> StmClosureResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.one_link(trans, lhs_dart_id, rhs_dart_id),
            2 => self.two_link(trans, lhs_dart_id, rhs_dart_id),
            3 => self.three_link(trans, lhs_dart_id, rhs_dart_id),
            _ => unreachable!(),
        }
    }

    /// `I`-unlink operator.
    ///
    /// # Description
    ///
    /// This operation corresponds to unlinking two darts by resetting their *β* images. Unlike
    /// unsewing, this does not alter associated attributes. For a thorough explanation of this
    /// operation, its hypothesis & consequences, refer to the [user guide][UG].
    ///
    /// [UG]: https://lihpc-computational-geometry.github.io/honeycomb/user-guide/definitions/sew.html
    ///
    /// # Arguments
    ///
    /// - `const I: u8` -- Unlink dimension.
    /// - `trans: &mut Transaction` -- Transaction associated to the operation.
    /// - `lhs_dart_id: DartIdType` -- First dart ID.
    ///
    /// The second dart ID is fetched using `I` and `lhs_dart_id`.
    ///
    /// # Errors
    ///
    /// This method should be called in a transactional context. The `Result` is then used to
    /// validate the transaction; Errors should not be processed manually, only processed via the
    /// `?` operator. The policy in case of failure can be defined when creating the transaction,
    /// using `Transaction::with_control`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - `I >= 4` or `I == 0`,
    /// - `lhs_dart_id` is already `I`-free.
    pub fn unlink<const I: u8>(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> StmClosureResult<()> {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.one_unlink(trans, lhs_dart_id),
            2 => self.two_unlink(trans, lhs_dart_id),
            3 => self.three_unlink(trans, lhs_dart_id),
            _ => unreachable!(),
        }
    }

    /// `I`-link operator.
    ///
    /// This variant is equivalent to [`link`][Self::link], but internally uses a transaction that
    /// will be retried until validated.
    pub fn force_link<const I: u8>(&self, lhs_dart_id: DartIdType, rhs_dart_id: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_link(lhs_dart_id, rhs_dart_id),
            2 => self.force_two_link(lhs_dart_id, rhs_dart_id),
            3 => self.force_three_link(lhs_dart_id, rhs_dart_id),
            _ => unreachable!(),
        }
    }

    /// `I`-unlink operator.
    ///
    /// This variant is equivalent to [`unlink`][Self::unlink], but internally uses a transaction
    /// that will be retried until validated.
    pub fn force_unlink<const I: u8>(&self, lhs_dart_id: DartIdType) {
        // these assertions + match on a const are optimized away
        assert!(I < 4);
        assert_ne!(I, 0);
        match I {
            1 => self.force_one_unlink(lhs_dart_id),
            2 => self.force_two_unlink(lhs_dart_id),
            3 => self.force_three_unlink(lhs_dart_id),
            _ => unreachable!(),
        }
    }
}
