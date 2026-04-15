/// 3D connected-component labeler for boolean cloud fields.
///
/// Rust port of the Fortran module `m_cloud_label` from
/// <https://github.com/jakubfabian/cloud_labeler>.
///
/// # Connectivity
/// 7-point stencil (6 face-neighbours + self):
/// * X and Y axes use **cyclic** (wrap-around) boundary conditions.
/// * Z axis uses **hard** (clamped) boundaries — no wrap-around.
///
/// # Flat index convention
/// Arrays are stored as `flat = x + nx*(y + ny*z)` — x is the
/// fastest-varying index, matching Fortran column-major order so that
/// the Python interface interoperates cleanly with the f2py Fortran wrapper.

// ─── Core algorithm ───────────────────────────────────────────────────────────

/// Cyclic index helper — mirrors the Fortran `cyclic(i, N)` pure function.
/// Maps any integer (including negative) into `[0, n)`.
#[inline]
pub fn cyclic(i: isize, n: usize) -> usize {
    i.rem_euclid(n as isize) as usize
}

/// Label every connected cloud patch in a 3-D boolean array.
///
/// # Arguments
/// * `cld`  — flat boolean array with index order `x + nx*(y + ny*z)`
///            (x is fastest — same as Fortran column-major)
/// * `nx`, `ny`, `nz` — array dimensions
///
/// # Returns
/// `Vec<Option<usize>>` of the same length.  `None` = non-cloud,
/// `Some(label)` = cloud cell (labels start at 0, increase in scan order).
pub fn gen_labels(cld: &[bool], nx: usize, ny: usize, nz: usize) -> Vec<Option<usize>> {
    assert_eq!(cld.len(), nx * ny * nz, "cld length must equal nx*ny*nz");

    let mut label: Vec<Option<usize>> = vec![None; nx * ny * nz];
    let idx = |x: usize, y: usize, z: usize| x + nx * (y + ny * z);

    let mut next_label: usize = 0;

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let i = idx(x, y, z);
                if cld[i] && label[i].is_none() {
                    flood_fill(x, y, z, nx, ny, nz, cld, &mut label, next_label, &idx);
                    next_label += 1;
                }
            }
        }
    }

    label
}

/// Iterative DFS flood-fill.  Replaces the original Fortran recursive
/// `fill_stencil` subroutine — no risk of stack-overflow on large domains.
fn flood_fill(
    x0: usize, y0: usize, z0: usize,
    nx: usize, ny: usize, nz: usize,
    cld: &[bool],
    label: &mut [Option<usize>],
    lval: usize,
    idx: &impl Fn(usize, usize, usize) -> usize,
) {
    let mut stack: Vec<(usize, usize, usize)> = vec![(x0, y0, z0)];

    while let Some((x, y, z)) = stack.pop() {
        let i = idx(x, y, z);
        if !cld[i] || label[i].is_some() { continue; }
        label[i] = Some(lval);

        // X / Y — cyclic boundary
        stack.push((cyclic(x as isize - 1, nx), y, z));
        stack.push((cyclic(x as isize + 1, nx), y, z));
        stack.push((x, cyclic(y as isize - 1, ny), z));
        stack.push((x, cyclic(y as isize + 1, ny), z));

        // Z — hard (clamped) boundary
        if z > 0       { stack.push((x, y, z - 1)); }
        if z + 1 < nz  { stack.push((x, y, z + 1)); }
    }
}

/// Count how many cells each label covers.
pub fn label_sizes(labels: &[Option<usize>]) -> Vec<usize> {
    match labels.iter().flatten().copied().max() {
        None    => vec![],
        Some(m) => {
            let mut counts = vec![0usize; m + 1];
            for &l in labels.iter().flatten() { counts[l] += 1; }
            counts
        }
    }
}

// ─── Python bindings (compiled only with --features python) ──────────────────

// PyO3 0.22 generates code that triggers the Rust-2024 unsafe_op_in_unsafe_fn
// lint inside its #[pyfunction] macros.  Suppress it until PyO3 0.23 fixes
// the generated code upstream.
#[cfg(feature = "python")]
#[allow(unsafe_op_in_unsafe_fn)]
mod python {
    use pyo3::prelude::*;
    use numpy::{
        ndarray::{Array3, ShapeBuilder},
        IntoPyArray, PyArray3, PyReadonlyArray3,
    };

    use super::gen_labels as rs_gen_labels;

    /// Label connected cloud patches.
    ///
    /// Parameters
    /// ----------
    /// cld : np.ndarray[bool, 3D]
    ///     Boolean cloud-mask array of shape (nx, ny, nz).
    ///     Any memory layout (C or Fortran order) is accepted.
    ///
    /// Returns
    /// -------
    /// np.ndarray[int32, 3D]
    ///     Label array of the same shape (Fortran-ordered).
    ///     -1  = not a cloud cell
    ///     ≥0  = patch label (0-based, in scan order x→y→z)
    #[pyfunction]
    fn gen_labels<'py>(
        py: Python<'py>,
        cld: PyReadonlyArray3<'py, bool>,
    ) -> PyResult<Bound<'py, PyArray3<i32>>> {
        let array = cld.as_array();
        let shape = array.shape();
        let (nx, ny, nz) = (shape[0], shape[1], shape[2]);

        // Build flat vec in Fortran order (x fastest) to match our index convention:
        //   flat = x + nx*(y + ny*z)
        //
        // Fast path: if the array is already F-contiguous (as numpy arrays created
        // with order='F' are), we can just clone the underlying memory slice —
        // a single memcpy instead of a triple-nested element loop.
        let cld_vec: Vec<bool> = if !array.is_standard_layout() {
            // F-order (or other non-C): get contiguous memory slice if available
            array.as_slice_memory_order()
                 .map(|s| s.to_vec())
                 .unwrap_or_else(|| array.iter().copied().collect())
        } else {
            // C-order: must reorder to put x as the innermost index
            let mut v = Vec::with_capacity(nx * ny * nz);
            for k in 0..nz {
                for j in 0..ny {
                    for i in 0..nx {
                        v.push(array[[i, j, k]]);
                    }
                }
            }
            v
        };

        let labels = rs_gen_labels(&cld_vec, nx, ny, nz);

        // Convert to i32 (-1 for None) and pack back into a Fortran-ordered array.
        let result_vec: Vec<i32> = labels.iter().map(|&l| match l {
            None    => -1i32,
            Some(v) => v as i32,
        }).collect();

        let result = Array3::from_shape_vec((nx, ny, nz).f(), result_vec)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        Ok(result.into_pyarray_bound(py))
    }

    /// Return the cell-count for each label.
    ///
    /// Parameters
    /// ----------
    /// labels : np.ndarray[int32, 3D]
    ///     Output of `gen_labels`.
    ///
    /// Returns
    /// -------
    /// list[int]
    ///     counts[i] = number of cells with label i.
    #[pyfunction]
    fn label_sizes<'py>(
        _py: Python<'py>,
        labels: PyReadonlyArray3<'py, i32>,
    ) -> Vec<usize> {
        let arr = labels.as_array();
        let max_label = arr.iter().copied().filter(|&v| v >= 0).max();
        match max_label {
            None    => vec![],
            Some(m) => {
                let mut counts = vec![0usize; m as usize + 1];
                for &v in arr.iter().filter(|&&v| v >= 0) {
                    counts[v as usize] += 1;
                }
                counts
            }
        }
    }

    /// Rust cloud_labeler Python extension.
    ///
    /// Functions
    /// ---------
    /// gen_labels(cld)   → label array
    /// label_sizes(labels) → list of cell counts per label
    #[pymodule]
    fn cloud_labeler_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(gen_labels, m)?)?;
        m.add_function(wrap_pyfunction!(label_sizes, m)?)?;
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(x: usize, y: usize, z: usize, nx: usize, ny: usize) -> usize {
        x + nx * (y + ny * z)
    }

    #[test] fn cyclic_interior()      { assert_eq!(cyclic(3, 10), 3); }
    #[test] fn cyclic_wrap_positive() { assert_eq!(cyclic(10, 10), 0); }
    #[test] fn cyclic_wrap_negative() { assert_eq!(cyclic(-1, 10), 9); }

    #[test]
    fn single_cloud_cell() {
        let nx = 3; let ny = 3; let nz = 1;
        let mut cld = vec![false; nx * ny * nz];
        cld[idx(1, 1, 0, nx, ny)] = true;
        let labels = gen_labels(&cld, nx, ny, nz);
        assert_eq!(labels[idx(1, 1, 0, nx, ny)], Some(0));
    }

    #[test] fn all_false_gives_all_none() {
        assert!(gen_labels(&vec![false; 27], 3, 3, 3).iter().all(|l| l.is_none()));
    }

    #[test] fn all_true_gives_single_label() {
        assert!(gen_labels(&vec![true; 27], 3, 3, 3).iter().all(|l| *l == Some(0)));
    }

    #[test]
    fn two_disconnected_cells_get_different_labels() {
        let nx = 3; let ny = 3; let nz = 1;
        let mut cld = vec![false; nx * ny * nz];
        cld[idx(0, 0, 0, nx, ny)] = true;
        cld[idx(2, 2, 0, nx, ny)] = true;
        let labels = gen_labels(&cld, nx, ny, nz);
        assert_ne!(labels[idx(0,0,0,nx,ny)], labels[idx(2,2,0,nx,ny)]);
    }

    #[test]
    fn cyclic_x_connects_opposite_edges() {
        let nx = 4; let ny = 3; let nz = 1;
        let mut cld = vec![false; nx * ny * nz];
        cld[idx(0, 1, 0, nx, ny)] = true;
        cld[idx(3, 1, 0, nx, ny)] = true;
        let labels = gen_labels(&cld, nx, ny, nz);
        assert_eq!(labels[idx(0,1,0,nx,ny)], labels[idx(3,1,0,nx,ny)]);
    }

    #[test]
    fn cyclic_y_connects_opposite_edges() {
        let nx = 3; let ny = 4; let nz = 1;
        let mut cld = vec![false; nx * ny * nz];
        cld[idx(1, 0, 0, nx, ny)] = true;
        cld[idx(1, 3, 0, nx, ny)] = true;
        let labels = gen_labels(&cld, nx, ny, nz);
        assert_eq!(labels[idx(1,0,0,nx,ny)], labels[idx(1,3,0,nx,ny)]);
    }

    #[test]
    fn z_axis_does_not_wrap() {
        let cld = vec![true, false, false, true];
        let labels = gen_labels(&cld, 1, 1, 4);
        assert_ne!(labels[0], labels[3]);
    }

    #[test]
    fn cross_pattern_is_single_component() {
        let (nx, ny, nz) = (10, 10, 1);
        let mut cld = vec![false; nx * ny * nz];
        for y in 1..=8 { cld[idx(4, y, 0, nx, ny)] = true; }
        for x in 1..=8 { cld[idx(x, 4, 0, nx, ny)] = true; }
        let labels = gen_labels(&cld, nx, ny, nz);
        let cloud_labels: Vec<usize> = labels.iter().flatten().copied().collect();
        assert!(cloud_labels.iter().all(|&l| l == cloud_labels[0]));
    }

    #[test]
    fn label_sizes_counts_correctly() {
        let nx = 3; let ny = 3; let nz = 1;
        let mut cld = vec![false; nx * ny * nz];
        cld[idx(0, 0, 0, nx, ny)] = true;
        cld[idx(1, 0, 0, nx, ny)] = true;
        cld[idx(2, 2, 0, nx, ny)] = true;
        let labels = gen_labels(&cld, nx, ny, nz);
        let sizes = label_sizes(&labels);
        assert_eq!(sizes, vec![2, 1]);
    }
}
