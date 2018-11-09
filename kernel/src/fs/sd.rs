use fat32::traits::BlockDevice;
use std::io;

extern "C" {
    /// A global representing the last SD controller error that occured.
    static sd_err: i64;

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occured, or -2 if an error sending commands to
    /// the SD controller occured.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occured or -2 if an
    /// error sending commands to the SD controller occured. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}

// Define a `#[no_mangle]` `wait_micros` function for use by `libsd`.
// The `wait_micros` C signature is: `void wait_micros(unsigned int);`
#[no_mangle]
pub extern "C" fn wait_micros(ms: u32) {
    ::pi::timer::spin_sleep_ms(ms as u64);
}

#[derive(Debug)]
pub enum Error {
    Timeout,
    SendingCommand,
    Other,
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    pub fn new() -> Result<Sd, Error> {
        let init_result = unsafe { sd_init() };
        match init_result {
            0 => Ok(Sd),
            -1 => Err(Error::Timeout),
            -2 => Err(Error::SendingCommand),
            _ => Err(Error::Other),
        }
    }
}

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 512 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "invalid buffer size {}, buffer must be at least 512 bytes",
                    buf.len()
                ),
            ));
        }
        if n > 2u64.pow(31) - 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid sector number {} (i32 overflow)", n),
            ));
        }

        let read = unsafe { sd_readsector(n as i32, buf.as_mut_ptr()) };

        if read == 0 {
            match unsafe { sd_err } {
                -1 => return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout")),
                _ => return Err(io::Error::new(io::ErrorKind::Other, "other error")),
            }
        } else {
            Ok(read as usize)
        }
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
