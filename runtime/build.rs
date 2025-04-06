//runtime/build.rs
#[cfg(feature = "std")]
fn main() {
    use substrate_wasm_builder::WasmBuilder;

    WasmBuilder::new()
        .with_current_project()
        .export_heap_base()
        .import_memory()
        .build();
}

#[cfg(not(feature = "std"))]
fn main() {
    // No-op for non-std environments
}
// runtime doesn't work 