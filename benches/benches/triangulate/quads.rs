// ------ IMPORTS

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use honeycomb::prelude::{
    triangulation::{earclip_cell, fan_cell, TriangulateError},
    CMap2, CMapBuilder, DartIdentifier, Orbit2, OrbitPolicy,
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
        .map(|id| (Orbit2::new(&map, OrbitPolicy::Face, *id as DartIdentifier).count() - 3) * 2)
        .collect();
    let n_tot: usize = n_darts_per_face.iter().sum();
    let tmp = map.add_free_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum: Vec<usize> = n_darts_per_face
        .iter()
        .enumerate()
        .map(|(i, _)| (0..i).map(|idx| n_darts_per_face[idx]).sum())
        .collect();
    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdentifier>> = n_darts_per_face
        .iter()
        .zip(prefix_sum.iter())
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdentifier..(tmp + start + n_d) as DartIdentifier)
                .collect::<Vec<_>>()
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
        .map(|id| (Orbit2::new(&map, OrbitPolicy::Face, *id as DartIdentifier).count() - 3) * 2)
        .collect();
    let n_tot: usize = n_darts_per_face.iter().sum();
    let tmp = map.add_free_darts(n_tot) as usize;
    // the prefix sum gives an offset that corresponds to the starting index of each slice, minus
    // the location of the allocated dart block (given by `tmp`)
    // end of the slice is deduced using these values and the number of darts the current seg needs
    let prefix_sum: Vec<usize> = n_darts_per_face
        .iter()
        .enumerate()
        .map(|(i, _)| (0..i).map(|idx| n_darts_per_face[idx]).sum())
        .collect();
    #[allow(clippy::cast_possible_truncation)]
    let dart_slices: Vec<Vec<DartIdentifier>> = n_darts_per_face
        .iter()
        .zip(prefix_sum.iter())
        .map(|(n_d, start)| {
            ((tmp + start) as DartIdentifier..(tmp + start + n_d) as DartIdentifier)
                .collect::<Vec<_>>()
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
