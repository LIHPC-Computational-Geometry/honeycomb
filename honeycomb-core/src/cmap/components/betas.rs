// ------ IMPORTS

use std::ops::{Index, IndexMut};

use stm::{StmError, TVar, Transaction};

use crate::cmap::NULL_DART_ID;

use super::identifiers::DartIdType;

// ------ CONTENT

/// Beta functions storage.
///
/// `N` is the number of beta function stored, including `B0`. This means that, for example,
/// a 2-map will have a `BetaFunctions<3>` object field.
pub struct BetaFunctions<const N: usize>(Vec<[TVar<DartIdType>; N]>);

/// Generate beta functions default value for a new dart.
fn new_beta_entry<const N: usize>() -> [TVar<DartIdType>; N] {
    (0..N)
        .map(|_| TVar::new(NULL_DART_ID))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

impl<const N: usize> BetaFunctions<N> {
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self((0..n_darts).map(|_| new_beta_entry()).collect())
    }

    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0.extend((0..len).map(|_| new_beta_entry()));
    }

    /// Return internal storage capacity
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl<const N: usize> Index<(u8, DartIdType)> for BetaFunctions<N> {
    type Output = TVar<DartIdType>;

    fn index(&self, (beta_id, dart_id): (u8, DartIdType)) -> &Self::Output {
        &self.0[dart_id as usize][beta_id as usize]
    }
}

impl<const N: usize> IndexMut<(u8, DartIdType)> for BetaFunctions<N> {
    fn index_mut(&mut self, (beta_id, dart_id): (u8, DartIdType)) -> &mut Self::Output {
        &mut self.0[dart_id as usize][beta_id as usize]
    }
}

// link methods

// unlink methods

impl<const N: usize> BetaFunctions<N> {
    /// 1-link operation.
    ///
    ///
    /// This operation corresponds to linking two darts via the *β<sub>1</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s). The *β<sub>0</sub>* function is also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Panics
    ///
    /// This method may panic if `lhs_dart_id` isn't 1-free or `rhs_dart_id` isn't 0-free.
    ///
    pub fn one_link_core(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        // we could technically overwrite the value, but these assertions
        // makes it easier to assert algorithm correctness
        assert!(self[(1, lhs_dart_id)].read_atomic() == NULL_DART_ID);
        assert!(self[(0, rhs_dart_id)].read_atomic() == NULL_DART_ID);
        // set beta_1(lhs_dart) to rhs_dart
        self[(1, lhs_dart_id)].write(trans, rhs_dart_id)?;
        // set beta_0(rhs_dart) to lhs_dart
        self[(0, rhs_dart_id)].write(trans, lhs_dart_id)?;
        Ok(())
    }

    /// 2-link operation.
    ///
    /// This operation corresponds to linking two darts via the *β<sub>2</sub>* function. Unlike
    /// its sewing counterpart, this method does not contain any code to update the attributes or
    /// geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rhs_dart_id: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` or `rhs_dart_id` isn't 2-free.
    pub fn two_link_core(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        // we could technically overwrite the value, but these assertions
        // make it easier to assert algorithm correctness
        assert!(self[(2, lhs_dart_id)].read_atomic() == NULL_DART_ID);
        assert!(self[(2, rhs_dart_id)].read_atomic() == NULL_DART_ID);
        // set beta_2(lhs_dart) to rhs_dart
        self[(2, lhs_dart_id)].write(trans, rhs_dart_id)?;
        // set beta_2(rhs_dart) to lhs_dart
        self[(2, rhs_dart_id)].write(trans, lhs_dart_id)?;
        Ok(())
    }

    /// 1-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>1</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s). The *β<sub>0</sub>* function is
    /// also updated.
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` is already 1-free.
    pub fn one_unlink_core(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        // set beta_1(lhs_dart) to NullDart
        let rhs_dart_id = self[(1, lhs_dart_id)].replace(trans, NULL_DART_ID)?;
        assert_ne!(rhs_dart_id, NULL_DART_ID);
        // set beta_0(rhs_dart) to NullDart
        self[(0, rhs_dart_id)].write(trans, NULL_DART_ID)?;
        Ok(())
    }

    /// 2-unlink operation.
    ///
    /// This operation corresponds to unlinking two darts that are linked via the *β<sub>2</sub>*
    /// function. Unlike its sewing counterpart, this method does not contain any code to update
    /// the attributes or geometrical data of the affected cell(s).
    ///
    /// # Arguments
    ///
    /// - `lhs_dart_id: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `lhs_dart_id` is already 2-free.
    pub fn two_unlink_core(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> Result<(), StmError> {
        // set beta_2(dart) to NullDart
        let rhs_dart_id = self[(2, lhs_dart_id)].replace(trans, NULL_DART_ID)?;
        assert_ne!(rhs_dart_id, NULL_DART_ID);
        // set beta_2(beta_2(dart)) to NullDart
        self[(2, rhs_dart_id)].write(trans, NULL_DART_ID)?;
        Ok(())
    }
}
