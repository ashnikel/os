use std::fmt;
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

const BIN_MIN_SIZE: usize = 3;
const BIN_MAX_SIZE: usize = 29; // 2^29=512M
// const BINS: usize = BIN_MAX_SIZE - BIN_MIN_SIZE + 1;
const BINS: usize = BIN_MAX_SIZE;

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    bins: [LinkedList; BINS]
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let mut bins = [LinkedList::new(); BINS];
        let mut start = start;

        while start < end {
            match Allocator::bin_fits_in_size(end - start) {
                Some(bin) => {
                    unsafe {
                        bins[bin].push(start as *mut usize);
                    }
                    start = start + 2usize.pow(bin as u32);
                }
                None => break,
            }
        }

        Allocator { bins }
    }

    fn bin_fits_in_size(size: usize) -> Option<usize> {
        for bin in BIN_MAX_SIZE..BIN_MIN_SIZE {
            if size >= 2usize.pow(bin as u32) {
                return Some(bin);
            }
        }
        None
    }

    fn bin_can_hold_size(size: usize) -> Option<usize> {
        let bin = size.next_power_of_two();

        if bin < BIN_MIN_SIZE {
            bin = BIN_MIN_SIZE;
        }

        if bin <= BIN_MAX_SIZE {
            Some(bin)
        } else {
            None
        }
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if !layout.align().is_power_of_two() {
            return Err(AllocErr::Unsupported {
                details: "layout alignment must be 2^n",
            });
        }

        if layout.size() == 0 {
            return Err(AllocErr::Unsupported {
                details: "allocating size cannot be equal 0",
            });
        }

        let max_size = layout.size() + layout.align();

        if let None = Allocator::bin_can_hold_size(max_size) {
            return Err(AllocErr::Exhausted { request: layout });
        }

        let fit_bin = Allocator::bin_can_hold_size(max_size).unwrap();

        // fitting bin available
        if let Some(_) = self.bins[fit_bin].peek() {
            return Ok(self.bins[fit_bin].pop().unwrap() as *mut u8);
        }

        // fitting bin not available, we nead to split larger bin
        for bin in fit_bin+1..BIN_MAX_SIZE {
            if let Some(_) = self.bins[bin].peek() {
                // bin can be split
                let mut bin_to_split = bin;
                while let None = self.bins[fit_bin].peek() {
                    Allocator::split_bin(self, bin_to_split);
                    bin_to_split -= 1;
                }
                break;
            }
        }

        match self.bins[fit_bin].peek() {
            Some(_) => Ok(self.bins[fit_bin].pop().unwrap() as *mut u8),
            None => return Err(AllocErr::Exhausted { request: layout }),
        }
    }

    fn split_bin(&mut self, bin: usize) {
        assert!(bin > BIN_MIN_SIZE);

        let addr = self.bins[bin].pop().unwrap();
        self.bins[bin - 1].push(addr);
        self.bins[bin - 1].push(addr.add(2usize.pow((bin - 1) as u32)));
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        unimplemented!("bin deallocation")
    }
}
//
// FIXME: Implement `Debug` for `Allocator`.
