#![feature(concat_idents)]
#![feature(decl_macro)]
#![feature(new_uninit)]
#![feature(offset_of)]
#![feature(slice_ptr_get)]

pub mod devdata;
pub mod devprop;
pub mod devset;
pub mod win;

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;

/// Allocate a slice of bytes with the given `size` and `align`ment
///
/// # Panic
///
/// This function can panic if the value of `align` is not a power of 2
/// of if the allocation fails
fn alloc_slice_with_align(size: NonZeroUsize, align: usize) -> Box<[MaybeUninit<u8>]> {
    use std::alloc::{alloc, handle_alloc_error};
    // TODO: maybe prefer `Layout::array()`?
    // let layout = Layout::array::<MaybeUninit<u8>>(size)
    //     .unwrap()
    //     .align_to(align)
    //     .unwrap();
    let layout = core::alloc::Layout::from_size_align(size.get(), align).unwrap();
    // SAFETY: from the safety section in docs of `core::alloc::GlobalAlloc::alloc()`
    // > undefined behavior can result if the caller does not ensure that layout has non-zero size
    // Given that `size` can't be 0, the `layout` is always valid
    let ptr = unsafe { alloc(layout) } as *mut MaybeUninit<u8>;
    if ptr.is_null() {
        handle_alloc_error(layout)
    }
    // SAFETY: from the safety section in the docs of `core::slice::from_raw_parts_mut()`
    // > - `data` must be valid for reads for `len * mem::size_of::<T>()` many bytes,
    // >   and it must be properly aligned. This means in particular:
    // >   - The entire memory range of this slice must be contained within a single
    // >     allocated object! [...].
    // >   - data must be non-null and aligned even for zero-length slices. [...].
    // The pointer returned by `alloc()` was checked for null and the alignment was checked
    // by `core::alloc::Layout::from_size_align()`
    // > - `data` must point to `len` consecutive properly initialized values of type `T`.
    // `MaybeUninit` allows for unitialized data
    // > - The memory referenced by the returned slice must not be mutated for the duration
    // >   of lifetime 'a [...].
    // From here on `ptr` is never accessed
    // > - The total size `len * mem::size_of::<T>()` of the slice must be no larger than `isize::MAX`.
    // This is guaranteed by `core::alloc::Layout::from_size_align()`
    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, size.into()) };
    // SAFETY: from the safety section in the docs of `std::boxed::Box::from_raw()`
    // > [...] It is valid to convert both ways between a Box and a raw pointer
    // > allocated with the Global allocator, given that the Layout used with the
    // > allocator is correct for the type. [...]
    // The layout is valid for a slice of u8s, and the pointer was returned by the global allocator
    unsafe { Box::from_raw(slice) }
}
