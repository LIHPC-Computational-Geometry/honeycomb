use honeycomb_render::{Runner, SmaaMode};

fn main() {
    Runner::default().run::<1, f32>(SmaaMode::Smaa1X, None);
}
