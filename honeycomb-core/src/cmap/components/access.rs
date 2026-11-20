//! Access controller definitions
//!
//! This module contains the [`AccessController`] trait, as well as provided implementations,
//! used to regulate how internal data of combinatorial maps are accessed. The controller type
//! is a generic parameter of the [`CMap2`][crate::cmap::CMap2] and
//! [`CMap3`][crate::cmap::CMap3] structures; its associated constants decide, at compile time,
//! whether each category of internal data is accessed through the transactional machinery or
//! through direct atomic operations.
//!
//! Because those constants are known at compile time, all branching done in access methods is
//! eliminated during monomorphization; the transactional (default) policy is therefore
//! cost-free with respect to the previous, non-generic implementation.

/// # Access controller trait
///
/// This trait describes, for each category of internal data of a combinatorial map, whether
/// accesses go through the transactional machinery (`true`) or bypass it using direct atomic
/// operations (`false`).
///
/// ## Categories
///
/// - [`BETAS_TX_ACCESS`][Self::BETAS_TX_ACCESS] -- *β* functions, i.e. the topological
///   information of the map,
/// - [`UNUSED_DARTS_TX_ACCESS`][Self::UNUSED_DARTS_TX_ACCESS] -- unused dart tracking flags,
/// - [`VERTICES_TX_ACCESS`][Self::VERTICES_TX_ACCESS] -- vertex embeddings,
/// - [`ATTRIBUTES_TX_ACCESS`][Self::ATTRIBUTES_TX_ACCESS] -- user-defined attributes.
///
/// ## Synchronization contract
///
/// - **Transactional accesses** (`true`) register reads and writes in the enclosing
///   [`Transaction`][crate::stm::Transaction], yielding conflict detection between concurrent
///   transactions and rollback of all operations in case of failure or retry.
/// - **Atomic accesses** (`false`) use the underlying shared variable's atomic read/write
///   operations directly. They provide no conflict detection and no rollback: writes performed
///   during a transaction that later fails are **not** reverted. Moreover, read-modify-write
///   patterns are composed of a read followed by a write, which is not atomic as a whole.
///
/// Because of this, a category configured for atomic access must not be accessed concurrently
/// while being modified, e.g. single-writer phases, read-only phases, or externally synchronized
/// sections of the user's algorithm. Under that assumption, atomic accesses are semantically
/// equivalent to their transactional counterparts, but cheaper.
pub trait AccessController: Send + Sync + 'static {
    /// Whether *β* function values are accessed transactionally.
    const BETAS_TX_ACCESS: bool;

    /// Whether unused dart tracking flags are accessed transactionally.
    const UNUSED_DARTS_TX_ACCESS: bool;

    /// Whether vertex embeddings are accessed transactionally.
    const VERTICES_TX_ACCESS: bool;

    /// Whether user-defined attribute values are accessed transactionally.
    const ATTRIBUTES_TX_ACCESS: bool;
}

/// # Provided controller -- fully transactional
///
/// All data categories are accessed through the transactional machinery. This is the default
/// policy of the map structures, and the behavior of pre-controller implementations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TransactionalController;

impl AccessController for TransactionalController {
    const BETAS_TX_ACCESS: bool = true;
    const UNUSED_DARTS_TX_ACCESS: bool = true;
    const VERTICES_TX_ACCESS: bool = true;
    const ATTRIBUTES_TX_ACCESS: bool = true;
}

/// # Provided controller -- fully atomic
///
/// All data categories bypass the transactional machinery and use direct atomic operations.
///
/// <div class="warning">
///
/// This controller provides **no conflict detection and no rollback**. Concurrent modifications
/// of a same data category can interleave and corrupt the map; read-modify-write sequences
/// (e.g. link operations checking freeness) are not atomic as a whole. This policy should only
/// be used when the user's algorithm guarantees that no concurrent access to the map's data
/// occurs, e.g. sequential workflows or externally synchronized phases.
///
/// </div>
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AtomicController;

impl AccessController for AtomicController {
    const BETAS_TX_ACCESS: bool = false;
    const UNUSED_DARTS_TX_ACCESS: bool = false;
    const VERTICES_TX_ACCESS: bool = false;
    const ATTRIBUTES_TX_ACCESS: bool = false;
}
