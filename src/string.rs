use std::borrow::{Borrow, Cow};
use std::ffi::CStr;
use std::fmt;
use std::ops::{Deref, Index, RangeFull};
use std::os::raw::c_char;
use std::str;

#[macro_export]
macro_rules! im_str {
    ($e:tt) => ({
        unsafe {
          $crate::ImStr::from_utf8_with_nul_unchecked(concat!($e, "\0").as_bytes())
        }
    });
    ($e:tt, $($arg:tt)*) => ({
        unsafe {
          &$crate::ImString::from_utf8_with_nul_unchecked(
            format!(concat!($e, "\0"), $($arg)*).into_bytes())
        }
    })
}

/// A UTF-8 encoded, growable, implicitly null-terminated string.
#[derive(Clone, Hash, Ord, Eq, PartialOrd, PartialEq)]
pub struct ImString(Vec<u8>);

impl ImString {
    /// Creates a new ImString from an existing string.
    pub fn new<T: Into<String>>(value: T) -> ImString {
        unsafe { ImString::from_utf8_unchecked(value.into().into_bytes()) }
    }
    pub fn with_capacity(capacity: usize) -> ImString {
        let mut v = Vec::with_capacity(capacity + 1);
        v.push(b'\0');
        ImString(v)
    }
    pub unsafe fn from_utf8_unchecked(mut v: Vec<u8>) -> ImString {
        v.push(b'\0');
        ImString(v)
    }
    pub unsafe fn from_utf8_with_nul_unchecked(v: Vec<u8>) -> ImString {
        ImString(v)
    }
    pub fn clear(&mut self) {
        self.0.clear();
        self.0.push(b'\0');
    }
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        self.push_str(ch.encode_utf8(&mut buf));
    }
    pub fn push_str(&mut self, string: &str) {
        self.refresh_len();
        self.0.extend_from_slice(string.as_bytes());
        self.0.push(b'\0');
    }
    pub fn capacity(&self) -> usize {
        self.0.capacity() - 1
    }
    pub fn capacity_with_nul(&self) -> usize {
        self.0.capacity()
    }
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr() as *const _
    }
    pub fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr() as *mut _
    }

    /// Updates the buffer length based on the current contents.
    ///
    /// Dear imgui accesses pointers directly, so the length doesn't get updated when the contents
    /// change. This is normally OK, because Deref to ImStr always calculates the slice length
    /// based on contents. However, we need to refresh the length in some ImString functions.
    fn refresh_len(&mut self) {
        let len = self.to_str().len();
        unsafe {
            self.0.set_len(len);
        }
    }
}

impl<'a> Default for ImString {
    fn default() -> ImString {
        unsafe { ImString::from_utf8_with_nul_unchecked(vec![0]) }
    }
}

impl From<String> for ImString {
    fn from(s: String) -> ImString {
        ImString::new(s)
    }
}

impl<'a> From<ImString> for Cow<'a, ImStr> {
    fn from(s: ImString) -> Cow<'a, ImStr> {
        Cow::Owned(s)
    }
}

impl<'a, T: ?Sized + AsRef<ImStr>> From<&'a T> for ImString {
    fn from(s: &'a T) -> ImString {
        s.as_ref().to_owned()
    }
}

impl AsRef<ImStr> for ImString {
    fn as_ref(&self) -> &ImStr {
        self
    }
}

impl Borrow<ImStr> for ImString {
    fn borrow(&self) -> &ImStr {
        self
    }
}

impl AsRef<str> for ImString {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl Borrow<str> for ImString {
    fn borrow(&self) -> &str {
        self.to_str()
    }
}

impl Index<RangeFull> for ImString {
    type Output = ImStr;
    fn index(&self, _index: RangeFull) -> &ImStr {
        self
    }
}

impl fmt::Debug for ImString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.to_str(), f)
    }
}

impl Deref for ImString {
    type Target = ImStr;
    fn deref(&self) -> &ImStr {
        // as_ptr() is used, because we need to look at the bytes to figure out the length
        // self.0.len() is incorrect, because there might be more than one nul byte in the end
        unsafe {
            &*(CStr::from_ptr(self.0.as_ptr() as *const c_char) as *const CStr as *const ImStr)
        }
    }
}

/// A UTF-8 encoded, implicitly null-terminated string slice.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImStr(CStr);

impl<'a> Default for &'a ImStr {
    fn default() -> &'a ImStr {
        static SLICE: &'static [u8] = &[0];
        unsafe { ImStr::from_utf8_with_nul_unchecked(SLICE) }
    }
}

impl fmt::Debug for ImStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl ImStr {
    pub fn new<S: AsRef<ImStr> + ?Sized>(s: &S) -> &ImStr {
        s.as_ref()
    }
    /// Converts a slice of bytes to an imgui-rs string slice without checking for valid UTF-8 or
    /// null termination.
    pub unsafe fn from_utf8_with_nul_unchecked(bytes: &[u8]) -> &ImStr {
        &*(bytes as *const [u8] as *const ImStr)
    }
    /// Converts a CStr reference to an imgui-rs string slice without checking for valid UTF-8.
    pub unsafe fn from_cstr_unchecked(value: &CStr) -> &ImStr {
        &*(value as *const CStr as *const ImStr)
    }
    /// Converts an imgui-rs string slice to a raw pointer
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }
    /// Converts an imgui-rs string slice to a normal string slice
    pub fn to_str(&self) -> &str {
        // CStr::to_bytes does *not* include the null terminator
        unsafe { str::from_utf8_unchecked(self.0.to_bytes()) }
    }
}

impl AsRef<CStr> for ImStr {
    fn as_ref(&self) -> &CStr {
        &self.0
    }
}

impl AsRef<ImStr> for ImStr {
    fn as_ref(&self) -> &ImStr {
        self
    }
}

impl AsRef<str> for ImStr {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl<'a> From<&'a ImStr> for Cow<'a, ImStr> {
    fn from(s: &'a ImStr) -> Cow<'a, ImStr> {
        Cow::Borrowed(s)
    }
}

impl ToOwned for ImStr {
    type Owned = ImString;
    fn to_owned(&self) -> ImString {
        ImString(self.0.to_owned().into_bytes())
    }
}
