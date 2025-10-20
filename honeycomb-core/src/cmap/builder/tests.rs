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
    //         map.force_read_vertex(d as VertexIdType),
    //         new_map.force_read_vertex(d as VertexIdType)
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
                map.force_read_vertex(d as VertexIdType),
                new_map.force_read_vertex(d as VertexIdType)
            );
        });
    }

    #[cfg(test)]
    const BAD_METAS: [&str; 9] = [
        "0.10.0 2",               // 2 elems
        "0.10.0 2 18 23",         // 4 elems
        "0.10.0 2.5 18",          // bad dim
        "0.10.0 2 hi",            // bad darts
        "0.10.0 bye 18",          // bad dim again
        "super super bad header", // ...
        "  ",                     // "" + ' ' + "" + ' ' + ""
        "",                       // empty
        "

             ",          // multiline
    ];

    #[cfg(test)]
    const MAP: &[u8] = b"[META]
0.10.0 2 4

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
