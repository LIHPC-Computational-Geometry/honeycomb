# Adding generic attributes to maps

---

## Entrypoint

The `attributes` module of the core crate provides the necessary tools for to add custom attributes
to given orbits of the map. Each attribute should be uniquely typed (i.e. to type aliases) as the
maps' internal storages use `std::any::TypeId` for identification. 

An attribute struct should implement both `AttributeBind` and `AttributeUpdate`. It can then be
added to the map using the dedicated `CMapBuilder` method.

## Example

### Implementing a `Weight` attribute

```rust
use honeycomb_core::attributes::{AttributeBind, AttributeUpdate, AttributeError, AttrSparseVec};
use honeycomb_core::cmap::{OrbitPolicy, VertexIdType};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Weight(pub u32);

impl AttributeUpdate for Weight {
    // when merging two weights, we add them 
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Self(attr1.0 + attr2.0))
    }

    // when splitting, we do an approximate 50/50
    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        // adding the % to keep things conservative
        Ok((Self(attr.0 / 2 + attr.0 % 2), Self(attr.0 / 2)))
    }

    // if we have to merge from a single value, we assume the "other" is 0
    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(attr)
    }
}

impl AttributeBind for Weight {
    // Weight values will be stored in an `AttrSparseVec`
    type StorageType = AttrSparseVec<Self>;
    // Weights bind to vertices 
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}
```


### Map integration

```rust
use honeycomb_core::cmap::{CMapBuilder, CMap2};

fn main() {
    let map: CMap2<_> = CMapBuilder::<2, f64>::from_n_darts(4)
        .add_attribute::<Weight>()
        .build()
        .unwrap();

    let _ = map.force_link::<2>(1, 2);
    let _ = map.force_link::<2>(3, 4);
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_write_vertex(3, (1.0, 1.0));
    map.force_write_attribute::<Weight>(2, Weight(5));
    map.force_write_attribute::<Weight>(3, Weight(6));

    let _ = map.force_sew::<1>(1, 3);

    assert_eq!(map.force_read_attribute::<Weight>(2), Some(Weight(11)));

    let _ = map.force_unsew::<1>(1);

    assert_eq!(map.force_read_attribute::<Weight>(2), Some(Weight(6)));
    assert_eq!(map.force_read_attribute::<Weight>(3), Some(Weight(5)));
}  
```


