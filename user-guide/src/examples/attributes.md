# Generic attribute system

---

The `attributes` module of the core crate provides the necessary tools for to add custom attributes
to given orbits of the map. Each attribute should be uniquely typed (i.e. no type aliases) as the
maps' internal storages use `std::any::TypeId` for identification.

An attribute struct should implement both `AttributeBind` and `AttributeUpdate`. It can then be
added to the map using the dedicated `CMapBuilder` method. This is showcased in n example below,
where we add 

### Implementation example

```rust
{{#include snippets/attribute_impl.rs}}
```

### Usage example

```rust
{{#include snippets/attribute_usage.rs}}
```

