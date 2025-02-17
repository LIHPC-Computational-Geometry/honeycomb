# (De)Serialization

---

Our crate implements two different serialization logics. We use a custom format for combinatorial
map representation, while we use the VTK format for mesh representation. The latter can also be
hijacked to initialize maps by restraining the range of supported data.

Serialization is available directly as map structures methods, while deserialization is available
through the usage of the builder structure.

## Custom serialization

### Format specification

The file should contain 4 sections:
- `[META]`: header with miscellaneous data to help parsing / checking
- `[BETAS]`: values of beta functions
- `[UNUSED]`: unused darts
- `[VERTICES]`: vertex identifiers and values

Single line comments are supproted using the `#` character.

#### META

Single line section specifying:
- format version,
- map dimension,
- number of darts.

The format version is the same as the crate's version; it serves as a hint to use the correct
crate version with a file you did not generate.


#### BETAS

Values of beta functions organized as one line per beta functions. The number of line should be
equal to dimension **plus one**.

All lines should:
- have the same length
- have a length equal the number of darts specified in the header **plus one** (for the null dart)

Values on a single line are separated by a space, and the first value of each line should be `0`
as they corresponds to beta images of the null dart.


#### UNUSED (optional)

Single line, optional section listing all unused darts. Unused darts should be free.


#### VERTICES

List of identifiers and corresponding vertex values. Vertices should have correct dimension (e.g.
x, y, and z coordinates for a 3-map).


### Examples

#### 2D

```toml
# unit square
[META] 
0.8.0 2 4 # <VERSION> <DIM> <N_DARTS_EXCL_0>

[BETAS] # 3 beta functions for 2D, 5 columns = 4 darts + the null dart
0 4 1 2 3 # b0 
0 1 3 4 1 # b1
0 0 0 0 0 # b2

[VERTICES]
1 0.0 0.0 # <ID> <X> <Y>
2 1.0 0.0
3 1.0 1.0
4 0.0 1.0
```


#### 3D

```toml
# simple tetrahedron
[META]
0.8.0 3 14

[BETAS] 
0 3 1 2 6 4 5 9 7 8 12 10 11 0 0 # columns don't have to be aligned,
0 2 3 1 5 6 4 8 9 7 11 12 10 0 0 # though our routines will align items
0 4 7 10 1 12 8 2 6 11 3 9 5 0 0

[UNUSED]
13 14

[VERTICES]
1 0.0 0.0 0.0 # <ID> <X> <Y> <Z>
2 1.0 0.0 0.0
3 1.0 1.0 0.0
6 0.5 0.5 1.0
```


## VTK serialization

We use the [`vtkio`](https://github.com/elrnv/vtkio) crate to handle file IO. Only the legacy
format is supported, in both its binary or ASCII form.


### Expected input for deserialization

Using a VTK file to initialize a map can fail for two main reason:

- The file contains general inconsistencies:
  - the number of coordinates cannot be divided by `3`, meaning a tuple is incomplete
  - the number of `Cells` and `CellTypes` isn't equal,
  - a given cell has an inconsistent number of vertices with its specified cell type.
- The file contains unsupported data:
  - file format isn't Legacy,
  - data set is something other than `UnstructuredGrid`,
  - coordinate representation type isn't `float` or `double`,
  - mesh contains unsupported cell types (`PolyVertex`, `PolyLine`, ...,  or anything 3D for
    a 2D map for example).
