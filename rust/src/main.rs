use cloud_labeler::{gen_labels, label_sizes};

fn main() {
    println!("=== Cloud Labeler — Rust demo ===");
    println!("Port of https://github.com/jakubfabian/cloud_labeler\n");

    // ── Cross pattern (mirrors Fortran test.f90) ──────────────────────────────
    println!("--- 10×10×1 cross pattern ---");
    {
        let (nx, ny, nz) = (10, 10, 1);
        let mut cld = vec![false; nx * ny * nz];
        let idx = |x: usize, y: usize, z: usize| x + nx * (y + ny * z);

        for y in 1..=8 { cld[idx(4, y, 0)] = true; }
        for x in 1..=8 { cld[idx(x, 4, 0)] = true; }

        let labels = gen_labels(&cld, nx, ny, nz);
        let sizes  = label_sizes(&labels);

        for y in 0..ny {
            let row: String = (0..nx).map(|x| match labels[idx(x, y, 0)] {
                None    => ". ".into(),
                Some(l) => format!("{} ", l),
            }).collect();
            print!("  {}", row);
        }
        println!();
        println!("\nFound {} patch(es):", sizes.len());
        for (l, &n) in sizes.iter().enumerate() { println!("  Label {l}: {n} cells"); }
    }

    println!();

    // ── Cyclic X boundary demo ────────────────────────────────────────────────
    println!("--- 6×4×1 — cyclic X connects opposite edges ---");
    {
        let (nx, ny, nz) = (6, 4, 1);
        let mut cld = vec![false; nx * ny * nz];
        let idx = |x: usize, y: usize, z: usize| x + nx * (y + ny * z);

        for y in 1..=2 { cld[idx(0, y, 0)] = true; cld[idx(5, y, 0)] = true; }
        cld[idx(2, 1, 0)] = true; cld[idx(3, 1, 0)] = true;

        let labels = gen_labels(&cld, nx, ny, nz);
        let sizes  = label_sizes(&labels);

        for y in 0..ny {
            let row: String = (0..nx).map(|x| match labels[idx(x, y, 0)] {
                None    => ". ".into(),
                Some(l) => format!("{} ", l),
            }).collect();
            print!("  {}", row);
        }
        println!("\n  (x=0 and x=5 are adjacent via cyclic X wrap)");
        println!("\nFound {} patch(es):", sizes.len());
        for (l, &n) in sizes.iter().enumerate() { println!("  Label {l}: {n} cells"); }
    }
}
