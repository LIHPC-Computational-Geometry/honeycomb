use std::ops::Range;

use fast_stm::{abort, atomically, atomically_with_err};

use crate::{
    cmap::{CMap2, CMap3, DartIdType},
    geometry::CoordsFloat,
};

/// Dart allocation or reservation error.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("failed dart allocation: {0}")]
pub struct DartAllocError(&'static str);

/// Pre-allocated dart fetching structure.
#[derive(Debug)]
pub struct CompactDartBlock<const SIZE: usize> {
    start: usize,
    cursor: usize,
}

impl<const SIZE: usize, T: CoordsFloat> TryFrom<&CMap2<T>> for CompactDartBlock<SIZE> {
    type Error = DartAllocError;

    fn try_from(map: &CMap2<T>) -> Result<Self, Self::Error> {
        for d in 1..=map.n_darts() - SIZE {
            if let Some(start) = atomically(|t| {
                let mut all_unused = true;
                for db in d..d + SIZE {
                    all_unused &= map.is_unused_transac(t, db as DartIdType)?;
                }

                if all_unused {
                    for db in d..d + SIZE {
                        map.set_used_transac(t, db as DartIdType)?;
                    }
                    return Ok(Some(d));
                }
                Ok(None)
            }) {
                return Ok(Self {
                    start,
                    cursor: start,
                });
            }
        }
        Err(DartAllocError("no contiguous unused block found"))
    }
}

impl<const SIZE: usize, T: CoordsFloat> TryFrom<&CMap3<T>> for CompactDartBlock<SIZE> {
    type Error = DartAllocError;

    fn try_from(map: &CMap3<T>) -> Result<Self, Self::Error> {
        for d in 1..map.n_darts() {
            if let Some(start) = atomically(|t| {
                let mut all_unused = true;
                for db in d..d + SIZE {
                    all_unused &= map.is_unused_transac(t, db as DartIdType)?;
                }

                if all_unused {
                    for db in d..d + SIZE {
                        map.set_used_transac(t, db as DartIdType)?;
                    }
                    return Ok(Some(d));
                }
                Ok(None)
            }) {
                return Ok(Self {
                    start,
                    cursor: start,
                });
            }
        }
        Err(DartAllocError("no contiguous unused block found"))
    }
}

impl<const SIZE: usize> CompactDartBlock<SIZE> {
    /// Take `n` free darts from the block.
    ///
    /// # Return
    ///
    /// This method will return:
    /// - `Some(r)` if there were enough darts left in the block,
    /// - `None` otherwise.
    #[must_use = "unused return value"]
    pub fn take_n(&mut self, n: usize) -> Option<Range<DartIdType>> {
        let c = self.cursor;
        if n != 0 && c + n <= self.start + SIZE {
            self.cursor += n;
            Some(c as DartIdType..self.cursor as DartIdType)
        } else {
            None
        }
    }

    /// Take all remaining dart from the block.
    ///
    /// # Return
    ///
    /// This method consumes the object and return all remaining darts in the block as a range.
    /// **Note that this range may be empty**.
    #[must_use = "unused return value"]
    pub fn take_remaining(self) -> Range<DartIdType> {
        self.cursor as DartIdType..(self.start + SIZE) as DartIdType
    }
}

/// Pre-allocated dart fetching structure.
pub struct SparseDartBlock<const SIZE: usize> {
    darts: [DartIdType; SIZE],
    cursor: usize,
}

impl<const SIZE: usize, T: CoordsFloat> TryFrom<&CMap2<T>> for SparseDartBlock<SIZE> {
    type Error = DartAllocError;

    fn try_from(map: &CMap2<T>) -> Result<Self, Self::Error> {
        atomically_with_err(|t| {
            let mut buf = Vec::new();
            for d in 1..map.n_darts() as DartIdType {
                if map.is_unused_transac(t, d)? {
                    map.set_used_transac(t, d)?;
                    buf.push(d);
                    if buf.len() == SIZE {
                        break;
                    }
                }
            }

            if buf.len() == SIZE {
                return Ok(Self {
                    darts: buf.try_into().expect("E: unreachable"), // safe due to outer if
                    cursor: 0,
                });
            }
            abort(DartAllocError(
                "not enough unused darts to build sparse block",
            ))
        })
    }
}

impl<const SIZE: usize, T: CoordsFloat> TryFrom<&CMap3<T>> for SparseDartBlock<SIZE> {
    type Error = DartAllocError;

    fn try_from(map: &CMap3<T>) -> Result<Self, Self::Error> {
        atomically_with_err(|t| {
            let mut buf = Vec::new();
            for d in 1..map.n_darts() as DartIdType {
                if map.is_unused_transac(t, d)? {
                    map.set_used_transac(t, d)?;
                    buf.push(d);
                    if buf.len() == SIZE {
                        break;
                    }
                }
            }

            if buf.len() == SIZE {
                return Ok(Self {
                    darts: buf.try_into().expect("E: unreachable"), // safe due to outer if
                    cursor: 0,
                });
            }
            abort(DartAllocError(
                "not enough unused darts to build sparse block",
            ))
        })
    }
}

impl<const SIZE: usize> SparseDartBlock<SIZE> {
    /// Take `n` free darts from the block.
    ///
    /// # Return
    ///
    /// This method will return:
    /// - `Some(slice)` if there were enough darts left in the block,
    /// - `None` otherwise.
    #[must_use = "unused return value"]
    pub fn take_n(&mut self, n: usize) -> Option<&[DartIdType]> {
        let c = self.cursor;
        if n != 0 && c + n <= SIZE {
            self.cursor += n;
            Some(&self.darts[c..self.cursor])
        } else {
            None
        }
    }

    /// Take all remaining dart from the block.
    ///
    /// # Return
    ///
    /// This method consumes the object and return all remaining darts in the block as a `Vec`.
    /// We have to use a dynamic collection to ensure we consume the object. **Note that the `Vec`
    /// may be empty**.
    #[must_use = "unused return value"]
    pub fn take_remaining(self) -> Vec<DartIdType> {
        self.darts[self.cursor..].to_vec()
    }
}
