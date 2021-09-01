use vergen::{Config, SemverKind, ShaKind, TimestampKind, vergen};

fn main() {
  // Setup the flags, toggling off the 'SEMVER_FROM_CARGO_PKG' flag
  let mut config = Config::default();
  *config.build_mut().kind_mut() = TimestampKind::All;
  *config.build_mut().semver_mut() = false;

  *config.git_mut().sha_kind_mut() = ShaKind::Short;
  *config.git_mut().semver_kind_mut() = SemverKind::Lightweight;

  // Generate the 'cargo:' key output
  vergen(config).expect("Unable to generate the cargo keys!");
}
