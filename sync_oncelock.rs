use spin::once::Once;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct OnceLock<T> {
    once: Once,
    value: core::cell::UnsafeCell<core::mem::MaybeUninit<T>>,
    initialized: AtomicBool,
}

unsafe impl<T> Send for OnceLock<T> {}
unsafe impl<T> Sync for OnceLock<T> {}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        OnceLock {
            once: Once::new(),
            value: core::cell::UnsafeCell::new(core::mem::MaybeUninit::uninit()),
            initialized: AtomicBool::new(false),
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.initialized.load(Ordering::Acquire) {
            Some(unsafe { &*(*self.value.get()).as_ptr() })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.initialized.load(Ordering::Acquire) {
            Some(unsafe { &mut *(*self.value.get()).as_mut_ptr() })
        } else {
            None
        }
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if let Some(val) = self.get() {
            return val;
        }
        self.once.call_once(|| {
            unsafe {
                (*self.value.get()).as_mut_ptr().write(f());
            }
            self.initialized.store(true, Ordering::Release);
        });
        unsafe { &*(*self.value.get()).as_ptr() }
    }

    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(unsafe { &*(*self.value.get()).as_ptr() });
        }
        let mut result: Option<Result<T, E>> = None;
        self.once.call_once(|| {
            match f() {
                Ok(val) => {
                    unsafe {
                        (*self.value.get()).as_mut_ptr().write(val);
                    }
                    self.initialized.store(true, Ordering::Release);
                    result = None;
                }
                Err(e) => {
                    result = Some(Err(e));
                }
            }
        });
        if self.initialized.load(Ordering::Acquire) {
            Ok(unsafe { &*(*self.value.get()).as_ptr() })
        } else {
            Err(result.unwrap().unwrap_err())
        }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        if self.initialized.load(Ordering::Acquire) {
            return Err(value);
        }
        let mut val = Some(value);
        self.once.call_once(|| {
            unsafe {
                (*self.value.get()).as_mut_ptr().write(val.take().unwrap());
            }
            self.initialized.store(true, Ordering::Release);
        });
        if self.initialized.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(val.take().unwrap())
        }
    }

    pub fn into_inner(self) -> Option<T> {
        if self.initialized.load(Ordering::Acquire) {
            Some(unsafe { (*self.value.into_inner()).assume_init() })
        } else {
            None
        }
    }

    pub fn take(&mut self) -> Option<T> {
        if self.initialized.load(Ordering::Acquire) {
            self.initialized.store(false, Ordering::Release);
            Some(unsafe { (*self.value.get()).as_mut_ptr().read() })
        } else {
            None
        }
    }
}

impl<T> core::fmt::Debug for OnceLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OnceLock")
    }
}

impl<T: Clone> Clone for OnceLock<T> {
    fn clone(&self) -> Self {
        OnceLock::new()
    }
}

impl<T> From<T> for OnceLock<T> {
    fn from(value: T) -> Self {
        let lock = OnceLock::new();
        lock.set(value).ok();
        lock
    }
}
