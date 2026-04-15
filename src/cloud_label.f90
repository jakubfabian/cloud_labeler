module m_cloud_label
  implicit none

  character(len=*), parameter :: help = &
    "Module to determine the index sets for cloud labels in a 3D Field"
  integer, parameter :: nil = -1

contains

  ! -- Public entry point ------------------------------------------------------
  subroutine gen_labels(cld, label)
    logical, intent(in)  :: cld(:,:,:)
    integer, intent(out) :: label(:,:,:)

    integer :: i, j, k, ilabel, Nx, Ny, Nz
    integer, allocatable :: stack(:)

    Nx = size(cld, 1);  Ny = size(cld, 2);  Nz = size(cld, 3)
    label = nil
    ilabel = 0

    ! Allocate the DFS stack once per call rather than once per component.
    ! Worst-case depth: every neighbour of every cell pushed before any pop
    ! -> 6 * (total cells).
    allocate(stack(6 * Nx * Ny * Nz))

    do k = 1, Nz
      do j = 1, Ny
        do i = 1, Nx
          if (cld(i,j,k) .and. label(i,j,k) == nil) then
            call fill_iterative(i, j, k, Nx, Ny, Nz, cld, ilabel, label, stack)
            ilabel = ilabel + 1
          end if
        end do
      end do
    end do

    deallocate(stack)
  end subroutine

  ! -- Iterative DFS flood-fill -------------------------------------------------
  ! Replaces the original recursive fill_stencil.
  !
  ! Cell coordinates are packed into a single flat 1-based integer to keep
  ! the stack as a plain integer array (better cache use, no tuple overhead):
  !
  !   flat = i  +  Nx*(j-1)  +  Nx*Ny*(k-1)
  !
  ! Changes vs the recursive version:
  !   * No per-cell subroutine call overhead or implicit call-stack frames.
  !   * Nx/Ny/Nz and NxNy computed once, not on every "recursive" entry.
  !   * Cyclic neighbours use a single conditional (merge) instead of two
  !     integer modulo operations per neighbour.
  subroutine fill_iterative(i0, j0, k0, Nx, Ny, Nz, cld, ilabel, label, stack)
    integer, intent(in)    :: i0, j0, k0, Nx, Ny, Nz, ilabel
    logical, intent(in)    :: cld(Nx, Ny, Nz)
    integer, intent(inout) :: label(Nx, Ny, Nz)
    integer, intent(inout) :: stack(:)

    integer :: top, flat, i, j, k, NxNy
    integer :: im1, ip1, jm1, jp1

    NxNy = Nx * Ny
    top  = 1
    stack(1) = i0 + Nx*(j0-1) + NxNy*(k0-1)

    do while (top > 0)
      flat = stack(top);  top = top - 1

      ! Decode 1-based (i, j, k) from flat index
      k = (flat - 1) / NxNy  + 1
      j = (flat - 1 - NxNy*(k-1)) / Nx + 1
      i =  flat - Nx*(j-1) - NxNy*(k-1)

      if (.not. cld(i,j,k) .or. label(i,j,k) /= nil) cycle

      label(i,j,k) = ilabel

      ! X neighbours - cyclic boundary
      im1 = merge(Nx, i-1, i == 1)
      ip1 = merge(1,  i+1, i == Nx)
      ! Y neighbours - cyclic boundary
      jm1 = merge(Ny, j-1, j == 1)
      jp1 = merge(1,  j+1, j == Ny)

      top = top + 1;  stack(top) = im1 + Nx*(j-1)   + NxNy*(k-1)
      top = top + 1;  stack(top) = ip1 + Nx*(j-1)   + NxNy*(k-1)
      top = top + 1;  stack(top) = i   + Nx*(jm1-1) + NxNy*(k-1)
      top = top + 1;  stack(top) = i   + Nx*(jp1-1) + NxNy*(k-1)
      if (k > 1)  then;  top = top + 1;  stack(top) = i + Nx*(j-1) + NxNy*(k-2);  end if
      if (k < Nz) then;  top = top + 1;  stack(top) = i + Nx*(j-1) + NxNy*k;      end if
    end do
  end subroutine

  ! Kept for the Python f2py interface and backwards compatibility.
  pure integer function cyclic(i, N)
    integer, intent(in) :: i, N
    cyclic = modulo(modulo(i-1, N) + N, N) + 1
  end function

end module
