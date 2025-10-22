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
{{#include snippets/attribute_impl.rs}}
```


### Map integration

```rust
{{#include snippets/attribute_usage.rs}}
```


