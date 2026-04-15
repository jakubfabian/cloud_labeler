message(STATUS "Configuring Rust cloud_labeler")

find_program(CARGO_EXECUTABLE cargo)
if(NOT CARGO_EXECUTABLE)
  message(FATAL_ERROR
    "cargo not found. Install Rust from https://rustup.rs\n"
    "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
endif()
message(STATUS "Found cargo: ${CARGO_EXECUTABLE}")

set(RUST_MANIFEST  ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml)
set(RUST_TARGET_DIR ${CMAKE_CURRENT_BINARY_DIR}/target)

# ── Rust library + demo binary (always built) ─────────────────────────────────
add_custom_target(cloud_labeler_rust ALL
  COMMAND ${CARGO_EXECUTABLE} build --release
          --manifest-path ${RUST_MANIFEST}
          --target-dir    ${RUST_TARGET_DIR}
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  COMMENT "Building Rust cloud_labeler (release)"
)

install(PROGRAMS ${RUST_TARGET_DIR}/release/cloud_labeler
        DESTINATION bin
        RENAME cloud_labeler_rs)

# ── Python extension (only when -DENABLE_PYTHON=YES) ─────────────────────────
if(ENABLE_PYTHON)
  # cargo build --features python  compiles PyO3 + numpy into the cdylib.
  # The output is  libcloud_labeler.so (Linux) / libcloud_labeler.dylib (macOS).
  # Python needs it as  cloud_labeler_rs.so  to match the #[pymodule] name.
  set(RUST_CDYLIB_LINUX ${RUST_TARGET_DIR}/release/libcloud_labeler.so)
  set(RUST_CDYLIB_MAC   ${RUST_TARGET_DIR}/release/libcloud_labeler.dylib)
  set(RUST_PYEXT_DIR    ${CMAKE_BINARY_DIR}/python)
  set(RUST_PYEXT_DST    ${RUST_PYEXT_DIR}/cloud_labeler_rs.so)

  add_custom_target(cloud_labeler_rust_python ALL
    # Build with the python feature flag
    COMMAND ${CARGO_EXECUTABLE} build --release --features python
            --manifest-path ${RUST_MANIFEST}
            --target-dir    ${RUST_TARGET_DIR}
    # Copy + rename so Python can import it
    COMMAND ${CMAKE_COMMAND} -E make_directory ${RUST_PYEXT_DIR}
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<IF:$<PLATFORM_ID:Darwin>,${RUST_CDYLIB_MAC},${RUST_CDYLIB_LINUX}>
            ${RUST_PYEXT_DST}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Building Rust Python extension → ${RUST_PYEXT_DST}"
    DEPENDS cloud_labeler_rust
  )
endif()
