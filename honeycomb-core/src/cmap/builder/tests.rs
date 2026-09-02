use vtkio::Vtk;

use crate::attributes::AttrStorageManager;
use crate::cmap::{CMap2, CMap3, CMapBuilder, DartIdType, OrbitPolicy};

// --- basic

#[test]
fn example_test() {
    let builder_2d = CMapBuilder::<2>::from_n_darts(10);
    let cmap_2d: CMap2<f64> = builder_2d.build().unwrap();
    assert_eq!(cmap_2d.n_darts(), 11);

    let builder_3d = CMapBuilder::<3>::from_n_darts(10);
    let cmap_3d: CMap3<f64> = builder_3d.build().unwrap();
    assert_eq!(cmap_3d.n_darts(), 11);
}

// --- IO

// ------ CMAP

#[cfg(test)]
mod cmap {
    use crate::cmap::{BuilderError, VertexIdType};

    use super::super::io::{CMapFile, build_2d_from_cmap_file, parse_meta};
    use super::*;

    #[test]
    fn bad_headers() {
        assert!(
            parse_meta(BAD_METAS[0])
                .is_err_and(|e| e == BuilderError::BadMetaData("incorrect format"))
        );
        assert!(
            parse_meta(BAD_METAS[1])
                .is_err_and(|e| e == BuilderError::BadMetaData("incorrect format"))
        );
        assert!(
            parse_meta(BAD_METAS[2])
                .is_err_and(|e| e == BuilderError::BadMetaData("could not parse dimension"))
        );
        assert!(
            parse_meta(BAD_METAS[3])
                .is_err_and(|e| e == BuilderError::BadMetaData("could not parse dart number"))
        );
        assert!(
            parse_meta(BAD_METAS[4])
                .is_err_and(|e| e == BuilderError::BadMetaData("could not parse dimension"))
        );
        assert!(
            parse_meta(BAD_METAS[5])
                .is_err_and(|e| e == BuilderError::BadMetaData("incorrect format"))
        );
        assert!(parse_meta(BAD_METAS[6]).is_err());
        assert!(parse_meta(BAD_METAS[7]).is_err());
        assert!(parse_meta(BAD_METAS[8]).is_err());
    }

    // #[test]
    // fn wr_identity() {
    // let map: CMap2<f32> = CMapBuilder::<2>::unit_grid(1).build().unwrap();
    // let mut buff = String::new();
    // map.serialize(&mut buff);
    // let cmap_file = CMapFile::try_from(buff).unwrap();
    // let new_map: CMap2<f32> =
    //     build_2d_from_cmap_file(cmap_file, AttrStorageManager::default()).unwrap();

    // assert_eq!(map.n_darts(), new_map.n_darts());
    // (0..map.n_darts() as DartIdType).for_each(|d| {
    //     assert_eq!(map.beta::<0>(d), new_map.beta::<0>(d));
    //     assert_eq!(map.beta::<1>(d), new_map.beta::<1>(d));
    //     assert_eq!(map.beta::<2>(d), new_map.beta::<2>(d));
    //     assert_eq!(
    //         map.read_vertex(d as VertexIdType),
    //         new_map.read_vertex(d as VertexIdType)
    //     );
    // });
    // }

    #[test]
    fn rwr_identity() {
        // deserialize -> serialize -> deserialize
        // check value consistency at each transformation
        let in_file = String::from_utf8(MAP.to_vec()).unwrap();
        let cmap_file = CMapFile::try_from(in_file.clone()).unwrap();
        let map: CMap2<f32> =
            build_2d_from_cmap_file(cmap_file, AttrStorageManager::default()).unwrap();

        let mut buff = String::new();
        map.serialize(&mut buff);

        assert_eq!(in_file.as_str(), buff.as_str());

        let cmap_file = CMapFile::try_from(buff).unwrap();
        let new_map: CMap2<f32> =
            build_2d_from_cmap_file(cmap_file, AttrStorageManager::default()).unwrap();
        assert_eq!(map.n_darts(), new_map.n_darts());
        (0..map.n_darts() as DartIdType).for_each(|d| {
            assert_eq!(map.beta::<0>(d), new_map.beta::<0>(d));
            assert_eq!(map.beta::<1>(d), new_map.beta::<1>(d));
            assert_eq!(map.beta::<2>(d), new_map.beta::<2>(d));
            assert_eq!(
                map.read_vertex(d as VertexIdType),
                new_map.read_vertex(d as VertexIdType)
            );
        });
    }

    #[cfg(test)]
    const BAD_METAS: [&str; 9] = [
        "0.11.0 2",               // 2 elems
        "0.11.0 2 18 23",         // 4 elems
        "0.11.0 2.5 18",          // bad dim
        "0.11.0 2 hi",            // bad darts
        "0.11.0 bye 18",          // bad dim again
        "super super bad header", // ...
        "  ",                     // "" + ' ' + "" + ' ' + ""
        "",                       // empty
        "

             ",          // multiline
    ];

    #[cfg(test)]
    const MAP: &[u8] = b"[META]
0.11.0 2 4

[BETAS]
0 4 1 2 3
0 2 3 4 1
0 0 0 0 0

[UNUSED]


[VERTICES]
1 0 0
2 1 0
3 1 1
4 0 1
";
}

// ------ VTK

#[cfg(test)]
mod vtk {
    use super::super::io::build_2d_from_vtk;
    use super::*;

    #[test]
    fn io_read() {
        let vtk = Vtk::parse_legacy_be(VTK_ASCII).unwrap();
        // unwrap is fine since we know the VTK_ASCII const is correct
        let cmap: CMap2<f32> = build_2d_from_vtk(vtk, AttrStorageManager::default()).unwrap();

        // check result
        let faces: Vec<_> = cmap.iter_faces().collect();
        assert_eq!(faces.len(), 4);
        assert_eq!(cmap.iter_edges().count(), 12);
        assert_eq!(cmap.iter_vertices().count(), 9);

        let mut n_vertices_per_face: Vec<usize> = faces
            .iter()
            .map(|id| cmap.orbit(OrbitPolicy::Face, *id as DartIdType).count())
            .collect();
        let (mut three_count, mut four_count, mut six_count): (usize, usize, usize) = (0, 0, 0);
        while let Some(n) = n_vertices_per_face.pop() {
            match n {
                3 => three_count += 1,
                4 => four_count += 1,
                6 => six_count += 1,
                _ => panic!("cmap was built incorrectly"),
            }
        }
        assert_eq!(three_count, 2);
        assert_eq!(four_count, 1);
        assert_eq!(six_count, 1);
    }

    #[cfg(test)]
    const VTK_ASCII: &[u8] = b"
# vtk DataFile Version 2.0
cmap
ASCII

DATASET UNSTRUCTURED_GRID
POINTS 9 float
0 0 0  1 0 0  1 1 0
0 1 0  2 0 0  2 1 0
2 2 0  1 3 0  0 2 0

CELLS 17 54
1 0
1 4
1 6
1 7
1 8
2 0 1
2 3 0
2 1 4
2 4 5
2 5 6
2 6 7
2 7 8
2 8 3
4 0 1 2 3
3 1 4 5
3 1 5 2
6 3 2 5 6 7 8

CELL_TYPES 17
1
1
1
1
1
3
3
3
3
3
3
3
3
9
5
5
7


POINT_DATA 9

CELL_DATA 17
";
}

// ------ Abaqus INP

#[cfg(test)]
mod inp {
    // use std::path::PathBuf;

    use crate::cmap::BuilderError;
    use crate::geometry::Vertex3;

    use super::super::io::build_3d_from_inp;
    use super::*;

    fn build(input: &str) -> Result<CMap3<f64>, BuilderError> {
        build_3d_from_inp(input, AttrStorageManager::default())
    }

    #[test]
    fn single_hex() {
        let map = build(SINGLE_HEX).unwrap();

        assert_eq!(map.n_darts(), 25);
        assert_eq!(map.iter_vertices().count(), 8);
        assert_eq!(map.iter_edges().count(), 12);
        assert_eq!(map.iter_faces().count(), 6);
        assert_eq!(map.iter_volumes().count(), 1);
        assert_eq!(
            map.iter_vertices()
                .filter_map(|vertex| map.read_vertex(vertex))
                .count(),
            8
        );
        assert_eq!(
            map.read_vertex(map.vertex_id(1)),
            Some(Vertex3(0.0, 0.0, 0.0))
        );
        assert_eq!(
            map.read_vertex(map.vertex_id(12)),
            Some(Vertex3(1.0, 1.0, 1.0))
        );
    }

    #[test]
    fn adjacent_hexes_are_sewn_using_connectivity() {
        let map = build(TWO_HEXES).unwrap();

        assert_eq!(map.n_darts(), 49);
        assert_eq!(map.iter_vertices().count(), 12);
        assert_eq!(map.iter_edges().count(), 20);
        assert_eq!(map.iter_faces().count(), 11);
        assert_eq!(map.iter_volumes().count(), 2);
        assert_eq!(
            map.iter_faces()
                .filter(|face| map.is_i_free::<3>(*face))
                .count(),
            10
        );
    }

    #[test]
    fn parses_sparse_ids_multiple_blocks_and_ignored_sections() {
        let input = TWO_HEXES
            .replace("*ELEMENT, TYPE=C3D8R", "*ELEMENT, ELSET=ONE, TYPE=c3d8r")
            .replace("2, 2, 9, 10, 3, 6, 11, 12, 7", "*MATERIAL, NAME=IGNORED\n*ELASTIC\n1, 2\n*ELEMENT, TYPE=C3D8, ELSET=TWO\n200, 2, 9, 10, 3, 6, 11, 12, 7");
        let map = build(&input).unwrap();

        assert_eq!(map.iter_vertices().count(), 12);
        assert_eq!(map.iter_volumes().count(), 2);
    }

    #[test]
    fn rejects_bad_or_unsupported_mesh_data() {
        assert_eq!(
            build(SINGLE_HEX.replace("C3D8R", "C3D4").as_str())
                .err()
                .unwrap(),
            BuilderError::UnsupportedInpData("only C3D8 element types are supported")
        );
        assert_eq!(
            build(SINGLE_HEX.replace(", 8\n", ", 99\n").as_str())
                .err()
                .unwrap(),
            BuilderError::BadInpData("element references an undefined node")
        );

        let same_orientation =
            format!("{SINGLE_HEX}\n*ELEMENT, TYPE=C3D8R\n2, 1, 2, 3, 4, 5, 6, 7, 8\n");
        assert_eq!(
            build(&same_orientation).err().unwrap(),
            BuilderError::BadInpData("adjacent elements have inconsistent face orientations")
        );

        let non_manifold = format!(
            "{TWO_HEXES}\n*NODE\n13, 0, 0, 2\n14, 1, 0, 2\n15, 1, 1, 2\n16, 0, 1, 2\n*ELEMENT, TYPE=C3D8R\n3, 2, 3, 7, 6, 13, 14, 15, 16\n"
        );
        assert_eq!(
            build(&non_manifold).err().unwrap(),
            BuilderError::BadInpData("face is shared by more than two elements")
        );
    }

    // #[test]
    // fn supplied_sphere_meshes() {
    //     for file_name in ["sphere_res_1cm.inp", "sphere_res_1cm_noised.inp"] {
    //         let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //             .parent()
    //             .unwrap()
    //             .join(file_name);
    //         let map: CMap3<f64> = CMapBuilder::<3>::from_inp_file(path).build().unwrap();

    //         assert_eq!(map.n_darts(), 245_761);
    //         assert_eq!(map.n_vertices(), 11_065);
    //         assert_eq!(
    //             (1..map.n_darts() as DartIdType)
    //                 .filter(|dart| map.is_i_free::<3>(*dart))
    //                 .count(),
    //             6_144
    //         );
    //     }
    // }

    const SINGLE_HEX: &str = "
*HEADING
ignored
** a comment
*NODE, NSET=ALL
1, 0, 0, 0
2, 1, 0, 0
3, 1, 1, 0
4, 0, 1, 0
5, 0, 0, 1
6, 1, 0, 1
7, 1, 1, 1
8, 0, 1, 1
*ELEMENT, TYPE=C3D8R
1, 1, 2, 3, 4, 5, 6, 7, 8
*SOLID SECTION, ELSET=ALL, MATERIAL=IGNORED
";

    const TWO_HEXES: &str = "
*NODE
1, 0, 0, 0
2, 1, 0, 0
3, 1, 1, 0
4, 0, 1, 0
5, 0, 0, 1
6, 1, 0, 1
7, 1, 1, 1
8, 0, 1, 1
9, 2, 0, 0
10, 2, 1, 0
11, 2, 0, 1
12, 2, 1, 1
*ELEMENT, TYPE=C3D8R
1, 1, 2, 3, 4, 5, 6, 7, 8
2, 2, 9, 10, 3, 6, 11, 12, 7
";
}
