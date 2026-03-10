use std::ops::{Index, IndexMut};

#[cfg(feature = "par-internals")]
use rayon::prelude::*;

use crate::cmap::{LinkError, NULL_DART_ID};
use crate::stm::{TVar, Transaction, TransactionClosureResult, abort};

use super::identifiers::DartIdType;

/// Beta functions storage.
///
/// `N` is the number of beta function stored, including `B0`. This means that, for example,
/// a 2-map will have a `BetaFunctions<3>` object field.
pub struct BetaFunctions<const N: usize>(Vec<[TVar<DartIdType>; N]>);

#[allow(unused)]
impl<const N: usize> BetaFunctions<N> {
    #[cfg(not(feature = "par-internals"))]
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self(
            (0..n_darts)
                .map(|_| std::array::from_fn(|_| TVar::new(NULL_DART_ID)))
                .collect(),
        )
    }

    #[cfg(feature = "par-internals")]
    /// Constructor
    pub fn new(n_darts: usize) -> Self {
        Self(
            (0..n_darts)
                .into_par_iter()
                .map(|_| std::array::from_fn(|_| TVar::new(NULL_DART_ID)))
                .collect(),
        )
    }

    #[cfg(not(feature = "par-internals"))]
    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0
            .extend((0..len).map(|_| std::array::from_fn(|_| TVar::new(NULL_DART_ID))));
    }

    #[cfg(feature = "par-internals")]
    /// Extend internal storage capacity
    pub fn extend(&mut self, len: usize) {
        self.0.par_extend(
            (0..len)
                .into_par_iter()
                .map(|_| std::array::from_fn(|_| TVar::new(NULL_DART_ID))),
        );
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
    /// - `ld: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rd: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Panics
    ///
    /// This method may panic if `ld` isn't 1-free or `rd` isn't 0-free.
    ///
    pub fn one_link_core(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        let b1ld = self[(1, ld)].replace(t, rd)?;
        let b0rd = self[(0, rd)].replace(t, ld)?;

        if b1ld != NULL_DART_ID {
            return abort(LinkError::NonFreeBase(1, ld, rd));
        }
        if b0rd != NULL_DART_ID {
            return abort(LinkError::NonFreeImage(0, ld, rd));
        }

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
    /// - `ld: DartIdentifier` -- ID of the first dart to be linked.
    /// - `rd: DartIdentifier` -- ID of the second dart to be linked.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `ld` or `rd` isn't 2-free.
    pub fn two_link_core(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        let b2ld = self[(2, ld)].replace(t, rd)?;
        let b2rd = self[(2, rd)].replace(t, ld)?;

        if b2ld != NULL_DART_ID {
            return abort(LinkError::NonFreeBase(2, ld, rd));
        }
        if b2rd != NULL_DART_ID {
            return abort(LinkError::NonFreeImage(2, ld, rd));
        }

        Ok(())
    }

    pub fn three_link_core(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        let b3ld = self[(3, ld)].replace(t, rd)?;
        let b3rd = self[(3, rd)].replace(t, ld)?;

        if b3ld != NULL_DART_ID {
            return abort(LinkError::NonFreeBase(3, ld, rd));
        }
        if b3rd != NULL_DART_ID {
            return abort(LinkError::NonFreeImage(3, ld, rd));
        }

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
    /// - `ld: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `ld` is already 1-free.
    pub fn one_unlink_core(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        // set beta_1(lhs_dart) to NullDart
        let rd = self[(1, ld)].replace(t, NULL_DART_ID)?;
        if rd == NULL_DART_ID {
            return abort(LinkError::AlreadyFree(1, ld));
        }
        // set beta_0(rhs_dart) to NullDart
        self[(0, rd)].write(t, NULL_DART_ID)?;
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
    /// - `ld: DartIdentifier` -- ID of the dart to unlink.
    ///
    /// # Panics
    ///
    /// This method may panic if one of `ld` is already 2-free.
    pub fn two_unlink_core(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        // set beta_2(dart) to NullDart
        let rd = self[(2, ld)].replace(t, NULL_DART_ID)?;
        if rd == NULL_DART_ID {
            return abort(LinkError::AlreadyFree(2, ld));
        }
        // set beta_2(beta_2(dart)) to NullDart
        self[(2, rd)].write(t, NULL_DART_ID)?;
        Ok(())
    }

    pub fn three_unlink_core(
        &self,
        t: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), LinkError> {
        // set beta_3(lhs_dart) to NullDart
        let rd = self[(3, ld)].replace(t, NULL_DART_ID)?;
        if rd == NULL_DART_ID {
            return abort(LinkError::AlreadyFree(3, ld));
        }
        // set beta_3(rhs_dart) to NullDart
        self[(3, rd)].write(t, NULL_DART_ID)?;
        Ok(())
    }
}
