use core::{
    fmt,
    hint::unreachable_unchecked,
    mem::{self, MaybeUninit},
};

/// A byte buffer that is incrementally filled and initialized.
///
/// It is ***undefined behavior*** to deinitialize previously
/// initialized data.
pub struct ReadBuf<'a> {
    /// The borroed byte buffer.
    buf: &'a mut [MaybeUninit<u8>],
    /// The amount of bytes that are known to be filled.
    ///
    /// This is always less than or equal to `self.init`.
    filled: usize,
    /// The amount of bytes that are known to be initialized.
    ///
    /// This is always less than or equal to `self.buf.len()`.
    init: usize,
}

impl<'a> ReadBuf<'a> {
    /// Method for ensuring that the invariants of this type are always met.
    ///
    /// In debug builds failure to verify the invariants of this type panic.
    ///
    /// In release builds, instead we tell the compiler that those invariants are always true,
    /// allowing for potential further optimizations.
    #[inline(always)]
    #[track_caller]
    unsafe fn _assert_invariants(&self) {
        if self.init > self.buf.len() {
            unsafe { _assert(Error::InitTooLarge.message()) }
        } else if self.filled > self.init {
            unsafe { _assert(Error::FilledTooLarge.message()) }
        }
    }
}

impl<'a> ReadBuf<'a> {
    /// Create a new buffer with all bytes within it being initialized and unfilled.
    #[inline]
    #[must_use]
    pub fn new(buf: &'a mut [u8]) -> Self {
        let init = buf.len();

        Self {
            buf: unsafe { slice_to_uninit_mut(buf) },
            filled: 0,
            init,
        }
    }

    /// Create a new buffer with all bytes within it being uninitialized and unfilled.
    #[inline]
    #[must_use]
    pub fn from_uninit(uninit: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            buf: uninit,
            filled: 0,
            init: 0,
        }
    }

    /// Get the capacity of the internal buffer.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn capacity(&self) -> usize {
        unsafe { self._assert_invariants() };
        self.buf.len()
    }

    /// Get the length of the filled buffer.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn filled_len(&self) -> usize {
        unsafe { self._assert_invariants() };
        self.filled
    }

    /// Get the length of the unfilled buffer.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn unfilled_len(&self) -> usize {
        unsafe { self._assert_invariants() };

        self.capacity() - self.filled
    }

    /// Get the length of the initialized buffer.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn init_len(&self) -> usize {
        unsafe { self._assert_invariants() };
        self.init
    }

    /// Get the length of the uninitalized buffer.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn uninit_len(&self) -> usize {
        unsafe { self._assert_invariants() };

        self.capacity() - self.init
    }

    /// Set the length of the filled buffer to zero.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn clear(&mut self) {
        unsafe { self._assert_invariants() };
        self.filled = 0;
    }

    /// Set the length of the filled buffer without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the new length of the filled
    /// buffer won't become larger than the initialized buffer.
    #[inline]
    #[track_caller]
    pub unsafe fn set_filled_unchecked(&mut self, filled: usize) {
        unsafe {
            self.filled = filled;
            self._assert_invariants();
        }
    }

    /// Try to set the length of the filled buffer.
    #[inline]
    #[track_caller]
    pub fn try_set_filled(&mut self, filled: usize) -> Result<(), Error> {
        if filled <= self.init {
            unsafe { self.set_filled_unchecked(filled) };

            Ok(())
        } else {
            Err(Error::FilledTooLarge)
        }
    }

    /// Set the length of the filled buffer.
    ///
    /// # Panics
    ///
    /// Panics if `filled` is larger than the size of the initialized buffer.
    #[inline]
    #[track_caller]
    pub fn set_filled(&mut self, filled: usize) {
        self.try_set_filled(filled).unwrap();
    }

    /// Advance the filled buffer by `n` bytes without checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the next `n` unfilled bytes are
    /// initialized.
    #[inline]
    #[track_caller]
    pub fn advance_unchecked(&mut self, n: usize) {
        let (filled, overflow) = self.filled.overflowing_add(n);

        if overflow {
            unsafe { _assert(Error::FilledTooLarge.message()) }
        }

        unsafe { self.set_filled_unchecked(filled) }
    }

    /// Try to advance the filled buffer by `n` bytes.
    #[inline]
    #[track_caller]
    pub fn try_advance(&mut self, n: usize) -> Result<(), Error> {
        match self.filled.checked_add(n) {
            Some(filled) => self.try_set_filled(filled),
            None => Err(Error::FilledTooLarge),
        }
    }

    /// Advance the filled buffer by `n` bytes.
    ///
    /// # Panics
    ///
    /// - The calculation of the new filled buffer length overflows.
    /// - The new filled buffer length exceeds the initialized buffer length.
    #[inline]
    #[track_caller]
    pub fn advance(&mut self, n: usize) {
        self.try_advance(n).unwrap()
    }

    /// Assert that the first `n` unfilled bytes are initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the first `n` unfilled bytes are initialized.
    #[inline]
    #[track_caller]
    pub unsafe fn assume_init(&mut self, n: usize) {
        unsafe { self._assert_invariants() };

        let new = self.filled + n;
        if new > self.init {
            self.init = new;
        }

        unsafe { self._assert_invariants() };
    }

    /// Get a slice of the filled buffer.
    #[inline]
    #[track_caller]
    pub fn filled(&self) -> &[u8] {
        unsafe {
            self._assert_invariants();
            uninit_to_slice(self.buf.get_unchecked(..self.filled))
        }
    }

    /// Get a mutable slice of the filled buffer.
    #[inline]
    #[track_caller]
    pub fn filled_mut(&mut self) -> &mut [u8] {
        unsafe {
            self._assert_invariants();
            uninit_to_slice_mut(self.buf.get_unchecked_mut(..self.filled))
        }
    }

    /// Get a slice of the unfilled buffer.
    #[inline]
    #[track_caller]
    pub fn unfilled(&self) -> &[MaybeUninit<u8>] {
        unsafe {
            self._assert_invariants();
            self.buf.get_unchecked(self.filled..)
        }
    }

    /// Get a mutable slice of the unfilled buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no bytes are deinitialized, including
    /// those that are already marked as uninitalized.
    #[inline]
    #[track_caller]
    pub unsafe fn unfilled_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            self._assert_invariants();
            self.buf.get_unchecked_mut(self.filled..)
        }
    }

    /// Get a slice of the initialized buffer.
    #[inline]
    #[track_caller]
    pub fn init(&self) -> &[u8] {
        unsafe {
            self._assert_invariants();
            uninit_to_slice(self.buf.get_unchecked(..self.init))
        }
    }

    /// Get a mutable slice of the initialized buffer.
    #[inline]
    #[track_caller]
    pub fn init_mut(&mut self) -> &mut [u8] {
        unsafe {
            self._assert_invariants();
            uninit_to_slice_mut(self.buf.get_unchecked_mut(..self.init))
        }
    }

    /// Get a slice of the uninitalized buffer.
    #[inline]
    #[track_caller]
    pub fn uninit(&self) -> &[MaybeUninit<u8>] {
        unsafe {
            self._assert_invariants();
            self.buf.get_unchecked(self.init..)
        }
    }

    /// Get a mutable slice of the uninitalized buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no bytes are deinitialized, including
    /// those that are already marked as uninitalized.
    #[inline]
    #[track_caller]
    pub unsafe fn uninit_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            self._assert_invariants();
            self.buf.get_unchecked_mut(self.init..)
        }
    }

    /// Initialize all uninitialized bytes with the provided value.
    ///
    /// This will update the initialized buffer length to span the entire
    /// internal buffer.
    ///
    /// # Returns
    ///
    /// Returns the now initialized slice.
    #[inline]
    #[track_caller]
    pub fn initialize_uninit(&mut self, byte: u8) -> &mut [u8] {
        // SAFETY: We only initialize values.
        let uninit = unsafe { self.uninit_mut() };

        uninit.fill(MaybeUninit::new(byte));

        let _ = uninit;

        let old_start = mem::replace(&mut self.init, self.buf.len());

        unsafe { uninit_to_slice_mut(self.buf.get_unchecked_mut(old_start..)) }
    }

    /// Initialize all unfilled bytes with the provided value.
    ///
    /// This will update the initialized buffer length to span the entire
    /// internal buffer.
    ///
    /// # Note
    ///
    /// This will also reinitialize already initialized unfilled bytes.
    /// This method does not distinguish between already initialized
    /// bytes and those that are not yet initialized.
    ///
    /// # Returns
    ///
    /// Returns the now fully initialized unfilled slice.
    #[inline]
    #[track_caller]
    pub fn initialize_unfilled(&mut self, byte: u8) -> &mut [u8] {
        // SAFETY: We only initialize values.
        let unfilled = unsafe { self.unfilled_mut() };

        unfilled.fill(MaybeUninit::new(byte));

        let _ = unfilled;

        self.init = self.buf.len();

        unsafe { uninit_to_slice_mut(self.unfilled_mut()) }
    }

    /// Try to push the contents of a slice into the buffer, updating the filled buffer length
    /// and potentially the initialized buffer length.
    #[inline]
    #[track_caller]
    pub fn try_push_slice(&mut self, slice: &[u8]) -> Result<(), Error> {
        // SAFETY: We only initialize data.
        let unfilled = unsafe { self.unfilled_mut() };

        let Some(unfilled) = unfilled.get_mut(..slice.len()) else {
            return Err(Error::SliceTooLarge);
        };

        // SAFETY: The length is checked above
        unsafe {
            unfilled
                .as_mut_ptr()
                .cast::<u8>()
                .copy_from_nonoverlapping(slice.as_ptr(), slice.len())
        }

        // Get rid of the slice reference just to make sure we don't accidentally fuck with it.
        let _ = unfilled;

        // SAFETY: This will never overflow as if it did we'd be unable to obtain a slice.
        let end = unsafe { self.filled.unchecked_add(slice.len()) };

        if self.init < end {
            self.init = end;
        }

        self.filled = end;

        // Just ensure that the invariants are met on debug builds.
        unsafe { self._assert_invariants() };

        Ok(())
    }

    /// Push the contents of a slice into the buffer, updating the filled buffer length
    /// and potentially the initialized buffer length.
    ///
    /// # Panics
    ///
    /// If the length of the slice exceeds the length of the unfilled buffer.
    #[inline]
    #[track_caller]
    pub fn push_slice(&mut self, slice: &[u8]) {
        self.try_push_slice(slice).unwrap();
    }
}

impl<'a> From<&'a mut [u8]> for ReadBuf<'a> {
    #[inline]
    fn from(value: &'a mut [u8]) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a mut [MaybeUninit<u8>]> for ReadBuf<'a> {
    #[inline]
    fn from(value: &'a mut [MaybeUninit<u8>]) -> Self {
        Self::from_uninit(value)
    }
}

impl fmt::Debug for ReadBuf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            f.debug_struct("ReadBuf")
                .field("filled", &self.filled)
                .field("initialized", &self.init)
                .field("capacity", &self.capacity())
                .finish()
        } else {
            f.debug_struct("ReadBuf")
                .field("filled", &self.filled())
                .field("initialized", &self.init)
                .field("capacity", &self.capacity())
                .finish()
        }
    }
}

#[inline]
#[must_use]
unsafe fn slice_to_uninit_mut<'a>(slice: &'a mut [u8]) -> &'a mut [MaybeUninit<u8>] {
    unsafe { &mut *(slice as *mut [u8] as *mut [MaybeUninit<u8>]) }
}

#[inline]
#[must_use]
unsafe fn uninit_to_slice_mut<'a>(uninit: &'a mut [MaybeUninit<u8>]) -> &'a mut [u8] {
    unsafe { &mut *(uninit as *mut [MaybeUninit<u8>] as *mut [u8]) }
}

#[inline]
#[must_use]
unsafe fn uninit_to_slice<'a>(uninit: &'a [MaybeUninit<u8>]) -> &'a [u8] {
    unsafe { &*(uninit as *const [MaybeUninit<u8>] as *const [u8]) }
}

#[inline(always)]
#[track_caller]
const unsafe fn _assert(message: &'static str) -> ! {
    if cfg!(debug_assertions) {
        panic!("{}", message)
    } else {
        unsafe { unreachable_unchecked() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    FilledTooLarge,
    InitTooLarge,
    SliceTooLarge,
}

impl Error {
    #[inline]
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Error::FilledTooLarge => {
                "filled buffer size exceeds the size of the initialized buffer"
            }
            Error::InitTooLarge => {
                "initialized buffer size exceeds the size of the internal buffer"
            }
            Error::SliceTooLarge => "there is not enough remaining room to append the slice",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
