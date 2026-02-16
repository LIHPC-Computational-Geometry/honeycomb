use coupe::{HilbertCurve, Partition, Point3D};
use honeycomb::prelude::{CoordsFloat, Vertex3};
use rand::{
    SeedableRng,
    distr::{Bernoulli, Distribution, Uniform},
    rngs::SmallRng,
};

pub type BRIO<T> = (Vec<Vertex3<T>>, Option<Vec<Vec<Vertex3<T>>>>);

/// Biased sort implementation
///
/// This function is loosely based on this paper: https://www.cs.ucdavis.edu/~amenta/pubs/brio.pdf
/// -- Incremental Contrsuctions con BRIO, N. Amenta et al
///
/// The main differences are as follow:
/// - TODO
pub fn compute_brio<T: CoordsFloat>(points: Vec<Point3D>, seed: u64) -> BRIO<T> {
    let seed = derive_seed(seed);
    let mut rng = SmallRng::seed_from_u64(seed);
    let dist = Bernoulli::new(0.3).unwrap();
    let n_rounds = (points.len().ilog2() + 1) as usize;

    // compute round attribution for all points

    let mut rounds_idx = vec![0; points.len()];
    (0..points.len()).for_each(|i| {
        let mut r = 0;
        while r < n_rounds - 1 && dist.sample(&mut rng) {
            r += 1;
        }
        rounds_idx[i] = n_rounds - 1 - r;
    });

    // build round collections

    let mut rounds = vec![vec![]; n_rounds];
    points
        .into_iter()
        .zip(rounds_idx.into_iter())
        .for_each(|(p, i)| {
            rounds[i].push(p);
        });

    // concatenate all first rounds with less than 2048 points for sequential execution
    // this will form round 1, next rounds will then use a dynamic number of threads for execution

    // find "up to which" base round we need to concatenate
    let concat_idx = if let Some(idx) = rounds
        .iter()
        .enumerate()
        .find_map(|(i, r)| if r.len() > 2048 { Some(i) } else { None })
    {
        idx
    } else {
        // this means that there isn't a single round with more than 2048 points
        // in this case, concatenate everything and run all in sequential
        n_rounds
    };

    // round 2 & up
    let n_threads = rayon::current_num_threads();
    let rs = rounds.split_off(concat_idx);

    // round 1
    let r1 = {
        let r: Vec<_> = rounds.into_iter().flat_map(|r| r.into_iter()).collect();
        let mut partition = vec![0; r.len()];
        let weights = vec![1.0; r.len()];
        // a Hilbert curve of order m contains 2^3m cells
        // choosing m = k*log2(n) allows to decouple the number of points
        // per cell from n
        HilbertCurve {
            part_count: 10,
            order: r.len().ilog2(),
        }
        .partition(&mut partition, (r.as_slice(), weights))
        .unwrap();

        let mut r: Vec<_> = r.into_iter().zip(partition).collect();
        r.sort_by(|(_, p_a), (_, p_b)| p_a.cmp(p_b));

        r.into_iter()
            .map(|(p, _)| {
                Vertex3(
                    T::from(p.x).unwrap(),
                    T::from(p.y).unwrap(),
                    T::from(p.z).unwrap(),
                )
            })
            .collect()
    };

    // sort all rounds

    let rs: Vec<_> = rs
        .into_iter()
        .map(|r| {
            let mut partition = vec![0; r.len()];
            let weights = vec![1.0; r.len()];
            HilbertCurve {
                part_count: 10 * n_threads,
                order: r.len().ilog2().div_ceil(2),
            }
            .partition(&mut partition, (r.as_slice(), weights))
            .unwrap();

            let mut r: Vec<_> = r.into_iter().zip(partition).collect();
            r.sort_by(|(_, p_a), (_, p_b)| p_a.cmp(p_b));

            r.into_iter()
                .map(|(p, _)| {
                    Vertex3(
                        T::from(p.x).unwrap(),
                        T::from(p.y).unwrap(),
                        T::from(p.z).unwrap(),
                    )
                })
                .collect()
        })
        .collect();

    (r1, if rs.is_empty() { None } else { Some(rs) })
}

/// SplitMix64 mixer
fn derive_seed(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

pub fn sample_points(
    lx: f64,
    ly: f64,
    lz: f64,
    n_points: usize,
    seed: u64,
) -> impl Iterator<Item = Point3D> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let xs: Vec<_> = {
        let dist = Uniform::try_from(0.0..lx).unwrap();
        dist.sample_iter(&mut rng).take(n_points).collect()
    };
    let ys: Vec<_> = {
        let dist = Uniform::try_from(0.0..ly).unwrap();
        dist.sample_iter(&mut rng).take(n_points).collect()
    };
    let zs: Vec<_> = {
        let dist = Uniform::try_from(0.0..lz).unwrap();
        dist.sample_iter(&mut rng).take(n_points).collect()
    };

    xs.into_iter()
        .zip(ys.into_iter().zip(zs))
        .map(|(x, (y, z))| Point3D::new(x, y, z))
}
