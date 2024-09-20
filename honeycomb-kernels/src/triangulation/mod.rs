mod ear_clipping;
mod fan;

pub use ear_clipping::process_cell as earclip_cell;
pub use fan::process_cell as fan_cell;

// ------ TESTS

#[cfg(test)]
mod tests;
