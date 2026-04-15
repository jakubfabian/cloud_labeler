! Timing benchmark for gen_labels — measures wall-clock time over many
! iterations so we can compare fairly against Criterion (Rust).
!
! Three workloads that match the Rust bench_fortran_test_equivalent group:
!   1. cross_10x10x1   — 10x10x1 cross pattern  (exact Fortran test.f90 size)
!   2. cross_100x100x1 — 100x100x1 cross pattern
!   3. cross_50x50x50  — 50x50x50 cross pattern
!
! Usage:
!   gfortran -O3 -o bench_cloud_label bench.f90 -L../lib -lcloud_label
!   ./bench_cloud_label

program bench
  use m_cloud_label
  implicit none

  integer :: i
  real(8) :: t0, t1, elapsed
  integer, parameter :: REPS_SMALL  = 100000
  integer, parameter :: REPS_MEDIUM = 10000
  integer, parameter :: REPS_LARGE  = 2000

  write(*,'(A)') "=== Fortran cloud_labeler benchmark ==="
  write(*,'(A)') "Workloads match Rust bench group: fortran_test_equivalent"
  write(*,*)

  ! ── 1. cross_10x10x1 ──────────────────────────────────────────────────────
  block
    integer, parameter :: Nx=10, Ny=10, Nz=1
    logical  :: cld(Nx,Ny,Nz)
    integer  :: label(Nx,Ny,Nz)
    cld = .False.
    cld(Nx/2,   2:Ny-1, :) = .True.
    cld(2:Nx-1, Ny/2,   :) = .True.
    call cpu_time(t0)
    do i = 1, REPS_SMALL
      call gen_labels(cld, label)
    enddo
    call cpu_time(t1)
    elapsed = (t1 - t0) / REPS_SMALL * 1.0d9  ! ns per iteration
    write(*,'(A,F10.1,A,I0,A)') &
      "cross_10x10x1   : ", elapsed, " ns/iter  (", REPS_SMALL, " reps)"
  end block

  ! ── 2. cross_100x100x1 ────────────────────────────────────────────────────
  block
    integer, parameter :: Nx=100, Ny=100, Nz=1
    logical  :: cld(Nx,Ny,Nz)
    integer  :: label(Nx,Ny,Nz)
    cld = .False.
    cld(Nx/2,   2:Ny-1, :) = .True.
    cld(2:Nx-1, Ny/2,   :) = .True.
    call cpu_time(t0)
    do i = 1, REPS_MEDIUM
      call gen_labels(cld, label)
    enddo
    call cpu_time(t1)
    elapsed = (t1 - t0) / REPS_MEDIUM * 1.0d6  ! µs per iteration
    write(*,'(A,F10.3,A,I0,A)') &
      "cross_100x100x1 : ", elapsed, " µs/iter  (", REPS_MEDIUM, " reps)"
  end block

  ! ── 3. cross_50x50x50 ─────────────────────────────────────────────────────
  block
    integer, parameter :: Nx=50, Ny=50, Nz=50
    logical  :: cld(Nx,Ny,Nz)
    integer  :: label(Nx,Ny,Nz)
    cld = .False.
    ! Vertical bar in x at mid_x, all y, all z
    cld(Nx/2,   2:Ny-1, :) = .True.
    ! Horizontal bar in y at mid_y, all x, all z
    cld(2:Nx-1, Ny/2,   :) = .True.
    call cpu_time(t0)
    do i = 1, REPS_LARGE
      call gen_labels(cld, label)
    enddo
    call cpu_time(t1)
    elapsed = (t1 - t0) / REPS_LARGE * 1.0d6  ! µs per iteration
    write(*,'(A,F10.3,A,I0,A)') &
      "cross_50x50x50  : ", elapsed, " µs/iter  (", REPS_LARGE, " reps)"
  end block

  write(*,*)
  write(*,'(A)') "Compare against Rust (cargo bench):"
  write(*,'(A)') "  fortran_test_equivalent/cross_10x10x1"
  write(*,'(A)') "  fortran_test_equivalent/cross_100x100x1"
  write(*,'(A)') "  fortran_test_equivalent/cross_50x50x50"

end program
