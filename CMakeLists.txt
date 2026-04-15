cmake_minimum_required (VERSION 2.8.10)

# guard against in-source builds
if(${CMAKE_SOURCE_DIR} STREQUAL ${CMAKE_BINARY_DIR})
  message(FATAL_ERROR "In-source builds not allowed. Please make a new directory (called a build directory) and run CMake from there.")
endif()

set(CMAKE_C_COMPILER   "gcc")
set(CMAKE_Fortran_COMPILER   "gfortran")

#set(CMAKE_Fortran_FLAGS "-cpp -ffree-line-length-none -g -fbacktrace -finit-real=nan -W -Wall -Wuninitialized -g -pg -fcheck=all -fbounds-check -pedantic -Wsurprising -ffpe-trap=invalid,zero,overflow -fno-range-check")
########################### Tenstream Project: ###################################
project (cloud_label C Fortran)

# ── Default build type ────────────────────────────────────────────────────────
if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release CACHE STRING
      "Build type: Release | Debug | RelWithDebInfo | MinSizeRel" FORCE)
  message(STATUS "CMAKE_BUILD_TYPE not set, defaulting to Release")
endif()

# ── Compiler optimizations ────────────────────────────────────────────────────
#
# NATIVE_ARCH (default ON)
#   Fortran : -march=native -funroll-loops
#             (gfortran equivalent of ifort/ifx -xHost)
#   Rust    : RUSTFLAGS="-C target-cpu=native"
#             (equivalent of Clang/LLVM -march=native)
#
# FAST_MATH (default OFF)
#   Fortran : -Ofast  (replaces -O3; enables -ffast-math and related
#             transforms — safe for pure-integer/boolean code, potentially
#             unsafe for code with strict IEEE-754 float requirements)
#   Rust    : -C opt-level=3 is unchanged; add "-C fast-math" for parity
#             (Rust stabilised fast-math per-operation, not globally)
#
# These are independent of CMAKE_BUILD_TYPE and stack on top of whatever
# flags that build type already provides.

option(NATIVE_ARCH
  "Optimise for the host CPU (gfortran: -march=native; Rust: target-cpu=native)"
  ON)

option(FAST_MATH
  "Enable -Ofast for Fortran (unsafe IEEE-754, harmless for integer code)"
  ON)

set(_FORT_EXTRA "")
if(NATIVE_ARCH)
  list(APPEND _FORT_EXTRA "-march=native" "-funroll-loops")
endif()
if(FAST_MATH)
  list(APPEND _FORT_EXTRA "-Ofast")
endif()

if(_FORT_EXTRA)
  string(JOIN " " _FORT_EXTRA_STR ${_FORT_EXTRA})
  set(CMAKE_Fortran_FLAGS "${CMAKE_Fortran_FLAGS} ${_FORT_EXTRA_STR}")
  message(STATUS "Fortran extra flags: ${_FORT_EXTRA_STR}")
endif()
set(CTEST_CMAKE_GENERATOR "Unix Makefiles")


# For python interfaces need -fPIC:
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Set Output Directories
set (CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib CACHE PATH "Output directory for Dohp archives")
set (CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib CACHE PATH "Output directory for Dohp libraries")
set (CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin CACHE PATH "Output directory for Dohp executables")

# Install Fortran *.mod files into include dir
set(CMAKE_Fortran_MODULE_DIRECTORY ${CMAKE_BINARY_DIR}/include )
install(DIRECTORY ${CMAKE_Fortran_MODULE_DIRECTORY}/ DESTINATION include)

# Set default install dir to local path
if (CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
  set (CMAKE_INSTALL_PREFIX "${CMAKE_BINARY_DIR}/install" CACHE PATH "default install path" FORCE )
endif()

# Add Source Directories
add_subdirectory(src)

# ── Rust implementation ───────────────────────────────────────────────────────
option(ENABLE_RUST "Build the Rust cloud_labeler implementation" ON)
if(ENABLE_RUST)
  add_subdirectory(rust)
endif()

# ── Python interfaces (Fortran f2py + Rust PyO3) ──────────────────────────────
option(ENABLE_PYTHON "Enable Python interfaces (f2py + Rust PyO3)" OFF)
if(ENABLE_PYTHON)
  find_program(F2PY_PATH f2py REQUIRED)
  message(STATUS "Found f2py: ${F2PY_PATH}")
  add_custom_target(python_interface ALL)
  add_subdirectory(python)
  if(ENABLE_RUST)
    # Ensure benchmark can find both .so files in the same directory
    message(STATUS "Python benchmark will be available at: ${CMAKE_BINARY_DIR}/python/benchmark.py")
  endif()
endif(ENABLE_PYTHON)
