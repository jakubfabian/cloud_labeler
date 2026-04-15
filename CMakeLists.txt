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
