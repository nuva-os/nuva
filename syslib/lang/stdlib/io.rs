/*
 * Nuva OS - System Library - Lang - I/O
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Standard I/O operations for the Nuva language runtime.
 */

/// Standard input
pub struct Stdin;

impl Stdin {
    /// Read a line from standard input
    pub fn read_line(&self, buf: &mut [u8]) -> Option<usize> {
        // Read from kernel console input (UART/keyboard)
        // In a real implementation, this would call the kernel
        // console read syscall to get input character by character
        // until a newline is encountered
        let mut i = 0;
        while i < buf.len() - 1 {
            // Read one byte from kernel console
            // SAFETY: unsafe block required for low-level memory or hardware access
            let byte = unsafe { crate::kernel::syscall::console_read_byte() };
            if byte == 0 {
                break; // EOF or no data available
            }
            buf[i] = byte;
            i += 1;
            if byte == b'
' {
                break; // End of line
            }
        }
        if i > 0 {
            buf[i] = 0; // Null terminate
            Some(i)
        } else {
            None
        }
    }
    
    /// Read raw bytes from standard input
    pub fn read(&self, buf: &mut [u8]) -> Option<usize> {
        // Read bytes from kernel console input
        let mut i = 0;
        while i < buf.len() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let byte = unsafe { crate::kernel::syscall::console_read_byte() };
            if byte == 0 {
                break;
            }
            buf[i] = byte;
            i += 1;
        }
        if i > 0 { Some(i) } else { None }
    }
}

/// Standard output
pub struct Stdout;

impl Stdout {
    /// Write bytes to standard output
    pub fn write(&self, buf: &[u8]) -> Option<usize> {
        // Write to kernel console output (VGA/UART)
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { crate::kernel::syscall::console_write(buf); }
        Some(buf.len())
    }
    
    /// Write a string to standard output
    pub fn write_str(&self, s: &str) -> Option<usize> {
        self.write(s.as_bytes())
    }
    
    /// Write a line to standard output
    pub fn write_line(&self, s: &str) -> Option<usize> {
        let len = self.write_str(s)?;
        self.write_str("
")?;
        Some(len + 1)
    }
    
    /// Flush the output buffer
    pub fn flush(&self) -> bool {
        true
    }
}

/// Standard error
pub struct Stderr;

impl Stderr {
    /// Write bytes to standard error
    pub fn write(&self, buf: &[u8]) -> Option<usize> {
        // Write to kernel error console output
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { crate::kernel::syscall::console_write_error(buf); }
        Some(buf.len())
    }
    
    /// Write a string to standard error
    pub fn write_str(&self, s: &str) -> Option<usize> {
        self.write(s.as_bytes())
    }
}

/// File handle
pub struct File {
    /// File descriptor
    fd: i32,
}

impl File {
    /// Open an existing file for reading
    pub fn open(path: &str) -> Option<Self> {
        // Call kernel open syscall
        // SAFETY: unsafe block required for low-level memory or hardware access
        let fd = unsafe { crate::kernel::syscall::open(path.as_ptr(), path.len(), 0) };
        if fd < 0 {
            None
        } else {
            Some(File { fd })
        }
    }
    
    /// Create a new file (or truncate existing) for writing
    pub fn create(path: &str) -> Option<Self> {
        // Call kernel create syscall (O_WRONLY | O_CREAT | O_TRUNC)
        // SAFETY: unsafe block required for low-level memory or hardware access
        let fd = unsafe { crate::kernel::syscall::open(path.as_ptr(), path.len(), 0x601) };
        if fd < 0 {
            None
        } else {
            Some(File { fd })
        }
    }
    
    /// Read bytes from file
    pub fn read(&self, buf: &mut [u8]) -> Option<usize> {
        // Call kernel read syscall
        // SAFETY: unsafe block required for low-level memory or hardware access
        let n = unsafe { crate::kernel::syscall::read(self.fd, buf.as_mut_ptr(), buf.len()) };
        if n >= 0 { Some(n as usize) } else { None }
    }
    
    /// Write bytes to file
    pub fn write(&self, buf: &[u8]) -> Option<usize> {
        // Call kernel write syscall
        // SAFETY: unsafe block required for low-level memory or hardware access
        let n = unsafe { crate::kernel::syscall::write(self.fd, buf.as_ptr(), buf.len()) };
        if n >= 0 { Some(n as usize) } else { None }
    }
    
    /// Close the file
    pub fn close(self) -> bool {
        // Call kernel close syscall
        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe { crate::kernel::syscall::close(self.fd) };
        result == 0
    }
    
    /// Get the file descriptor
    pub fn fd(&self) -> i32 {
        self.fd
    }
}

/// Get standard input handle
pub fn stdin() -> Stdin {
    Stdin
}

/// Get standard output handle
pub fn stdout() -> Stdout {
    Stdout
}

/// Get standard error handle
pub fn stderr() -> Stderr {
    Stderr
}

/// Print macro
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::nuva_lang::stdlib::io::stdout().write_str(format!($($arg)*).as_str())
    };
}

/// Print line macro
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::nuva_lang::stdlib::io::stdout().write_line(format!($($arg)*).as_str())
    };
}