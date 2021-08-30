use vergen::{Config, vergen};

fn main() {
  // Setup the flags, toggling off the 'SEMVER_FROM_CARGO_PKG' flag
  let mut config = Config::default();
  *config.build_mut().semver_mut() = false;

  // Generate the 'cargo:' key output
  vergen(config).expect("Unable to generate the cargo keys!");
}
