//node/build.rs
use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
    // Generate necessary cargo keys
    generate_cargo_keys();

    // Re-run the build script if there's a change in the Git HEAD
    rerun_if_git_head_changed();
}
