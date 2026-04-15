message(STATUS "Creating Python interfaces with ${F2PY_PATH}")

# Copy the benchmark script into the build/python directory at *build* time
# so that edits to the source are reflected after the next `make`.
add_custom_command(
  OUTPUT  ${CMAKE_CURRENT_BINARY_DIR}/benchmark.py
  COMMAND ${CMAKE_COMMAND} -E copy_if_different
          ${CMAKE_CURRENT_SOURCE_DIR}/benchmark.py
          ${CMAKE_CURRENT_BINARY_DIR}/benchmark.py
  DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/benchmark.py
  COMMENT "Copying benchmark.py to build directory"
)
add_custom_target(copy_benchmark_py ALL
  DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/benchmark.py
)

# ── f2py — Fortran Python extension ──────────────────────────────────────────
#
# Python 3.12 + numpy ≥ 1.25 use the meson backend instead of distutils.
# The meson backend does not support the old  -L<dir> -l<lib>  approach; it
# must compile all Fortran sources directly.  So we pass both the wrapper
# source (py_cloud_label.f90) AND the core algorithm source (cloud_label.f90)
# to f2py in one invocation.
#
# The output file is placed in CMAKE_CURRENT_BINARY_DIR (= build/python/).
# Its exact name depends on the Python version and platform, e.g.
#   py_cloud_label.cpython-312-x86_64-linux-gnu.so
# Python's import machinery finds it automatically when that directory is on
# sys.path.

add_custom_target(do_python_interface_py_cloud_label ALL
  COMMAND ${CMAKE_COMMAND} -E env CC=${CMAKE_C_COMPILER}
          ${F2PY_PATH}
          --f90exec=${CMAKE_Fortran_COMPILER}
          -m py_cloud_label
          -c ${CMAKE_CURRENT_SOURCE_DIR}/py_cloud_label.f90
             ${CMAKE_SOURCE_DIR}/src/cloud_label.f90
  WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
  DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/py_cloud_label.f90
          ${CMAKE_SOURCE_DIR}/src/cloud_label.f90
  COMMENT "Building f2py module 'py_cloud_label'"
)
add_dependencies(python_interface do_python_interface_py_cloud_label)
