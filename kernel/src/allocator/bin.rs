use std::fmt;
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;


use console::kprintln; ///////////////////////////////////////////////////////////////////////////////////////

const BIN_MIN_SIZE: usize = 3;
const BIN_MAX_SIZE: usize = 29; // 2^29=512M
const BINS: usize = BIN_MAX_SIZE + 1;

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    bins: [LinkedList; BINS],
    start: usize,
}

/// Return bin size (2^bin_num)
fn bin_size(bin_num: usize) -> usize {
    1 << bin_num
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let mut bins = [LinkedList::new(); BINS];
        let mut current = start;
        while current < end {
            match Allocator::max_bin_fits_in_size(end - current) {
                Some(bin) => {
                    unsafe {
                        bins[bin].push(current as *mut usize);
                    }
                    current += bin_size(bin);
                }
                None => break,
            }
        }
        Allocator {
            bins,
            start,
        }
    }

    fn max_bin_fits_in_size(size: usize) -> Option<usize> {
        for bin in (BIN_MIN_SIZE..BIN_MAX_SIZE).rev() {
            if size >= bin_size(bin) {
                kprintln!("*** bin_fits_in_size {} ***", bin);
                return Some(bin);
            }
        }
        None
    }

    fn bin_can_hold_size(size: usize) -> Option<usize> {
        for bin in BIN_MIN_SIZE..BIN_MAX_SIZE {
            if size <= bin_size(bin) {
                return Some(bin)
            }
        }
        None
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

        // guarantee that layout can be aligned in bin
        let size = layout.size() + layout.align();

        // size > size of MAX_BIN
        if let None = Allocator::bin_can_hold_size(size) {
            return Err(AllocErr::Exhausted { request: layout });
        }

        let fit_bin = Allocator::bin_can_hold_size(size).unwrap();

        // fitting bin available
        if !self.bins[fit_bin].is_empty() {
            let addr = self.bins[fit_bin].pop().unwrap();
            let fuck = addr as usize;
            let addru8 = addr as *mut u8;
            let fuck2 = addru8 as usize;
            let aligned_addr = align_up(addr as usize, layout.align());
            return Ok(aligned_addr as *mut u8);
            // return Ok(addru8);
        }

        // fitting bin not available, we nead to split larger bin
        for bin in fit_bin+1..BIN_MAX_SIZE {
            if !self.bins[bin].is_empty() {
                // bin can be split
                let mut bin_to_split = bin;
                while self.bins[fit_bin].is_empty() && bin_to_split > fit_bin {
                    if !self.bins[bin_to_split].is_empty() {
                        self.split_bin(bin_to_split);
                    }
                    bin_to_split -= 1;
                }
                break;
            }
        }

        if !self.bins[fit_bin].is_empty() {
            let addr = self.bins[fit_bin].pop().unwrap();
            let addru8 = addr as *mut u8;
            let aligned_addr = align_up(addr as usize, layout.align());
            return Ok(aligned_addr as *mut u8);
            // return Ok(addru8);
        } else {
            return Err(AllocErr::Exhausted { request: layout });
        }
    }

    fn split_bin(&mut self, bin: usize) {
        assert!(bin > BIN_MIN_SIZE);

        unsafe {
            let addr = self.bins[bin].pop().unwrap();
            let smaller_bin = bin - 1;
            self.bins[smaller_bin].push(addr as *mut usize);//////
            let buddy_addr = addr as usize + bin_size(smaller_bin);

            self.bins[smaller_bin].push(buddy_addr as *mut usize);
        }
    }

    fn merge_if_buddy_is_empty(&mut self, bin: usize, bin_addr: usize) -> Option<usize> {
        let buddy_addr = if ((bin_addr - self.start)/ bin_size(bin)) % 2 == 0 {
            // Left bin. Check buddy to the right
            bin_addr + bin_size(bin)
        } else {
            // Right bin. Check buddy to the left
            bin_addr - bin_size(bin)
        };

        let mut merged = false;

        for node in self.bins[bin].iter_mut() {
            if node.value() as usize == buddy_addr {
                node.pop();
                merged = true;
                break;
            }
        }

        unsafe {
            if merged {
                    if bin_addr < buddy_addr {
                        self.bins[bin + 1].push(bin_addr as *mut usize);
                        return Some(bin_addr);
                    } else {
                        self.bins[bin + 1].push(buddy_addr as *mut usize);
                        return Some(buddy_addr);

                    }
            } else {
                self.bins[bin].push(bin_addr as *mut usize);
                return None;
            }
        }
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
    unsafe {///////////////////////////////////////////1
        let size = layout.size() + layout.align();

        let wtf = ptr as usize;

        let fit_bin = Allocator::bin_can_hold_size(size).unwrap();

        let distance_to_start = (ptr as *mut usize) as usize - self.start;
        let shift_to_bin_start = distance_to_start % bin_size(fit_bin);
        let bin_addr = ptr as usize - shift_to_bin_start;


        let mut next_addr = bin_addr;
        for bin in fit_bin..BIN_MAX_SIZE {
            match self.merge_if_buddy_is_empty(bin, next_addr) {
                Some(addr) => next_addr = addr,
                None => break,
            }
        }
    }
    }///////////////////////////////////////////
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Allocator")
            .field("bin0 (must be empty)", &self.bins[0])
            .field("bin1 (must be empty)", &self.bins[1])
            .field("bin2 (must be empty)", &self.bins[2])
            .field("bin3 (8)", &self.bins[3])
            .field("bin4 (16)", &self.bins[4])
            .field("bin5 (32)", &self.bins[5])
            .field("bin6 (64)", &self.bins[6])
            .field("bin7 (128)", &self.bins[7])
            .field("bin8 (256)", &self.bins[8])
            .field("bin9 (512)", &self.bins[9])
            .field("bin10 (1K)", &self.bins[10])
            .field("bin11 (2K)", &self.bins[11])
            .field("bin12 (4K)", &self.bins[12])
            .field("bin13 (8K)", &self.bins[13])
            .field("bin14 (16K)", &self.bins[14])
            .field("bin15 (32K)", &self.bins[15])
            .field("bin16 (64K)", &self.bins[16])
            .field("bin17 (128K)", &self.bins[17])
            .field("bin18 (256K)", &self.bins[18])
            .field("bin19 (512K)", &self.bins[19])
            .field("bin20 (1M)", &self.bins[20])
            .field("bin21 (2M)", &self.bins[21])
            .field("bin22 (4M)", &self.bins[22])
            .field("bin23 (8M)", &self.bins[23])
            .field("bin24 (16M)", &self.bins[24])
            .field("bin25 (32M)", &self.bins[25])
            .field("bin26 (64M)", &self.bins[26])
            .field("bin27 (128M)", &self.bins[27])
            .field("bin28 (256M)", &self.bins[28])
            .field("bin29 (512M)", &self.bins[29])
            .finish()
    }
}