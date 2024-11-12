// ------ IMPORTS

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use honeycomb::prelude::{
    triangulation::{earclip_cell, fan_cell, TriangulateError},
    CMap2, CMapBuilder, DartIdType, Orbit2, OrbitPolicy,
};
use honeycomb_benches::FloatType;

// ------ CONTENT

const PATH: &str = "../examples/quads.vtk";

fn fan_bench() -> Result<(), TriangulateError> {
    let mut map: CMap2<FloatType> = CMapBuilder::default().vtk_file(PATH).build().unwrap();

    // prealloc darts
    let faces = map.fetch_faces().identifiers.clone();
    let n_darts_per_face: Vec<_> = faces
        .iter()
        .map(|id| (Orbit2::new(&map, OrbitPolicy::Face, *id as DartIdType).count() - 3) * 2)
        .collect();
    let n_tot: usize = n_darts_per_face.iter().sum();
    let tmp = map.add_free_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum = n_darts_per_face.iter().scan(0, |state, &n_d| {
        *state += n_d;
        Some(*state - n_d) // we want an offset, not the actual sum
    });
    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdType>> = n_darts_per_face
        .iter()
        .zip(prefix_sum)
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdType..(tmp + start + n_d) as DartIdType).collect::<Vec<_>>()
        })
        .collect();

    for (face_id, new_darts) in faces.iter().zip(dart_slices.iter()) {
        fan_cell(&mut map, *face_id, new_darts)?
    }

    Ok(())
}

fn earclip_bench() -> Result<(), TriangulateError> {
    let mut map: CMap2<FloatType> = CMapBuilder::default().vtk_file(PATH).build().unwrap();

    // prealloc darts
    let faces = map.fetch_faces().identifiers.clone();
    let n_darts_per_face: Vec<_> = faces
        .iter()
        .map(|id| (Orbit2::new(&map, OrbitPolicy::Face, *id as DartIdType).count() - 3) * 2)
        .collect();
    let n_tot: usize = n_darts_per_face.iter().sum();
    let tmp = map.add_free_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum = n_darts_per_face.iter().scan(0, |state, &n_d| {
        *state += n_d;
        Some(*state - n_d) // we want an offset, not the actual sum
    });
    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdType>> = n_darts_per_face
        .iter()
        .zip(prefix_sum)
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdType..(tmp + start + n_d) as DartIdType).collect::<Vec<_>>()
        })
        .collect();

    for (face_id, new_darts) in faces.iter().zip(dart_slices.iter()) {
        earclip_cell(&mut map, *face_id, new_darts)?
    }

    Ok(())
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("triangulation");

    group.bench_function("fan", |b| b.iter(|| black_box(fan_bench())));
    group.bench_function("earclip", |b| b.iter(|| black_box(earclip_bench())));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
