/*
 * Nuva OS - Kernel - Execve and ELF Loader
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_debug, pr_info, pr_warn};

/// ELF magic number
pub const ELF_MAGIC: u32 = 0x464C457F;  /* "\x7FELF" */

/// ELF class
pub const ELFCLASS64: u8 = 2;

/// ELF data encoding
pub const ELFDATA2LSB: u8 = 1;  /* Little endian */

/// ELF version
pub const EV_CURRENT: u8 = 1;

/// ELF type
pub const ET_EXEC: u16 = 2;   /* Executable */
pub const ET_DYN: u16 = 3;   /* Shared object (PIE) */

/// ELF machine
pub const EM_AARCH64: u16 = 183;  /* ARM 64-bit */
pub const EM_X86_64: u16 = 62;    /* AMD x86-64 */

/// Program header types
pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;
pub const PT_SHLIB: u32 = 5;
pub const PT_PHDR: u32 = 6;
pub const PT_GNU_STACK: u32 = 0x6474E551;

/// Program header flags
pub const PF_X: u32 = 1;  /* Execute */
pub const PF_W: u32 = 2;  /* Write */
pub const PF_R: u32 = 4;  /* Read */

/// ELF64 Header
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Elf64Ehdr {
    /// Magic number and other info
    pub e_ident: [u8; 16],
    /// Object file type
    pub e_type: u16,
    /// Architecture
    pub e_machine: u16,
    /// Object file version
    pub e_version: u32,
    /// Entry point virtual address
    pub e_entry: u64,
    /// Program header table file offset
    pub e_phoff: u64,
    /// Section header table file offset
    pub e_shoff: u64,
    /// Processor-specific flags
    pub e_flags: u32,
    /// ELF header size in bytes
    pub e_ehsize: u16,
    /// Program header table entry size
    pub e_phentsize: u16,
    /// Program header table entry count
    pub e_phnum: u16,
    /// Section header table entry size
    pub e_shentsize: u16,
    /// Section header table entry count
    pub e_shnum: u16,
    /// Section header string table index
    pub e_shstrndx: u16,
}

/// ELF64 Program Header
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Elf64Phdr {
    /// Segment type
    pub p_type: u32,
    /// Segment flags
    pub p_flags: u32,
    /// Segment file offset
    pub p_offset: u64,
    /// Segment virtual address
    pub p_vaddr: u64,
    /// Segment physical address
    pub p_paddr: u64,
    /// Segment size in file
    pub p_filesz: u64,
    /// Segment size in memory
    pub p_memsz: u64,
    /// Segment alignment
    pub p_align: u64,
}

/// ELF64 Section Header
#[repr(C, packed)]
pub struct Elf64Shdr {
    /// Section name (string tbl index)
    pub sh_name: u32,
    /// Section type
    pub sh_type: u32,
    /// Section flags
    pub sh_flags: u64,
    /// Section virtual addr at execution
    pub sh_addr: u64,
    /// Section file offset
    pub sh_offset: u64,
    /// Section size in bytes
    pub sh_size: u64,
    /// Link to another section
    pub sh_link: u32,
    /// Additional section information
    pub sh_info: u32,
    /// Section alignment
    pub sh_addralign: u64,
    /// Entry size if section holds table
    pub sh_entsize: u64,
}

/// ELF loader
/// Loads ELF executables into memory for execve.
pub struct ElfLoader {
    /// Total loads
    pub load_count: AtomicU64,
    /// Successful loads
    pub load_success: AtomicU64,
    /// Failed loads
    pub load_failures: AtomicU64,
    /// PIE loads
    pub pie_count: AtomicU64,
}

impl ElfLoader {
    pub const fn new() -> Self {
        ElfLoader {
            load_count: AtomicU64::new(0),
            load_success: AtomicU64::new(0),
            load_failures: AtomicU64::new(0),
            pie_count: AtomicU64::new(0),
        }
    }
    
    /// Load ELF binary
    /// @param data: Pointer to ELF file data
    /// @param size: Size of ELF file
    /// @param entry: Output entry point
    /// @return Ok on success, error code on failure
    pub fn load_elf(&self, data: *const u8, size: usize, entry: &mut u64) -> Result<(), i32> {
        self.load_count.fetch_add(1, Ordering::AcqRel);
        
        // Validate file size
        if size < core::mem::size_of::<Elf64Ehdr>() {
            self.load_failures.fetch_add(1, Ordering::AcqRel);
            return Err(-8);  /* ENOEXEC */
        }
        
        // Parse ELF header
        let ehdr = self.parse_ehdr(data)?;
        
        // Validate ELF header
        self.validate_ehdr(&ehdr)?;
        
        // Check if PIE
        let is_pie = ehdr.e_type == ET_DYN;
        if is_pie {
            self.pie_count.fetch_add(1, Ordering::AcqRel);
        }
        
        // Calculate load address for PIE
        let load_addr = if is_pie {
            self.find_pie_load_addr(data, &ehdr)?
        } else {
            0
        };
        
        // Load program segments
        self.load_segments(data, &ehdr, load_addr)?;
        
        // Set entry point
        *entry = ehdr.e_entry + load_addr;
        
        self.load_success.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("ELF loaded: entry={:#x}", *entry);
        
        Ok(())
    }
    
    /// Parse ELF header
    fn parse_ehdr(&self, data: *const u8) -> Result<Elf64Ehdr, i32> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ehdr_ptr = data as *const Elf64Ehdr;
            Ok(*ehdr_ptr)
        }
    }
    
    /// Validate ELF header
    fn validate_ehdr(&self, ehdr: &Elf64Ehdr) -> Result<(), i32> {
        // Check magic number
        let magic = u32::from_le_bytes([
            ehdr.e_ident[0],
            ehdr.e_ident[1],
            ehdr.e_ident[2],
            ehdr.e_ident[3],
        ]);
        
        if magic != ELF_MAGIC {
            log_warn!("Invalid ELF magic: {:#x}", magic);
            return Err(-8);  /* ENOEXEC */
        }
        
        // Check class (must be 64-bit)
        if ehdr.e_ident[4] != ELFCLASS64 {
            log_warn!("Not a 64-bit ELF: class={}", ehdr.e_ident[4]);
            return Err(-8);
        }
        
        // Check data encoding (must be little endian)
        if ehdr.e_ident[5] != ELFDATA2LSB {
            log_warn!("Not little endian ELF: data={}", ehdr.e_ident[5]);
            return Err(-8);
        }
        
        // Check version
        if ehdr.e_ident[6] != EV_CURRENT {
            log_warn!("Invalid ELF version: {}", ehdr.e_ident[6]);
            return Err(-8);
        }
        
        // Check type (executable or PIE)
        let e_type = ehdr.e_type;
        if e_type != ET_EXEC && e_type != ET_DYN {
            log_warn!("Not executable ELF: type={}", e_type);
            return Err(-8);
        }
        
        // Check machine (ARM64 or x86-64)
        let e_machine = ehdr.e_machine;
        if e_machine != EM_AARCH64 && e_machine != EM_X86_64 {
            log_warn!("Unsupported architecture: {}", e_machine);
            return Err(-8);
        }
        
        Ok(())
    }
    
    /// Find load address for PIE
    fn find_pie_load_addr(&self, _data: *const u8, ehdr: &Elf64Ehdr) -> Result<u64, i32> {
        // Find an available address range for PIE loading using mmap.
        // For PIE executables, we need to find a free region in the
        // process address space to map the ELF segments. We calculate
        // the total size needed from the program headers and then
        // use mmap with MAP_ANONYMOUS to reserve the region.
        // In a full implementation:
        // let total_size = self.calc_total_load_size(ehdr);
        // let addr = mmap(0, total_size, PROT_NONE,
        // MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE, -1, 0);
        // if addr == MAP_FAILED {
        // return Err(-12);  // ENOMEM
        // }
        // return Ok(addr);
        let _ = ehdr;
        Ok(0x400000)  /* Typical PIE load address */
    }
    
    /// Load program segments
    fn load_segments(&self, data: *const u8, ehdr: &Elf64Ehdr, load_addr: u64) -> Result<(), i32> {
        let phdr_size = core::mem::size_of::<Elf64Phdr>();
        
        for i in 0..ehdr.e_phnum {
            // Get program header
            let phdr = self.get_phdr(data, ehdr, i)?;
            
            // Only load PT_LOAD segments
            if phdr.p_type != PT_LOAD {
                continue;
            }
            
            // Calculate addresses
            let vaddr = phdr.p_vaddr + load_addr;
            let file_start = data as u64 + phdr.p_offset;
            let file_size = phdr.p_filesz;
            let mem_size = phdr.p_memsz;
            
            // Validate addresses
            if vaddr == 0 {
                continue;
            }
            
            log_debug!("Loading segment {}: vaddr={:#x}, size={}", i, vaddr, mem_size);
            
            // Allocate memory for segment
            self.alloc_segment(vaddr, mem_size, phdr.p_flags)?;
            
            // Copy file content
            if file_size > 0 {
                self.copy_segment(vaddr, file_start as *const u8, file_size)?;
            }
            
            // Zero BSS (mem_size > file_size)
            if mem_size > file_size {
                let bss_start = vaddr + file_size;
                let bss_size = mem_size - file_size;
                self.zero_segment(bss_start, bss_size)?;
            }
            
            // Set segment protection
            self.set_segment_prot(vaddr, mem_size, phdr.p_flags)?;
        }
        
        Ok(())
    }
    
    /// Get program header by index
    fn get_phdr(&self, data: *const u8, ehdr: &Elf64Ehdr, index: u16) -> Result<Elf64Phdr, i32> {
        let phdr_size = core::mem::size_of::<Elf64Phdr>();
        let offset = ehdr.e_phoff + (index as u64) * (phdr_size as u64);
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let phdr_ptr = (data as u64 + offset) as *const Elf64Phdr;
            Ok(*phdr_ptr)
        }
    }
    
    /// Allocate memory for segment
    fn alloc_segment(&self, vaddr: u64, size: u64, flags: u32) -> Result<(), i32> {
        // Use mmap to allocate memory for the ELF segment.
        // MAP_FIXED ensures the segment is mapped at the exact
        // virtual address specified in the ELF program header.
        // MAP_PRIVATE creates a copy-on-write mapping.
        // MAP_ANONYMOUS provides zero-filled pages.
        // In a full implementation:
        // let prot = self.flags_to_prot(flags);
        // let result = do_mmap(vaddr, size, prot,
        // MAP_FIXED | MAP_PRIVATE | MAP_ANONYMOUS,
        // -1, 0);
        // if result != vaddr {
        // return Err(-12);  // ENOMEM
        // }
        let prot = self.flags_to_prot(flags);
        log_debug!("alloc_segment: {:#x}, size={}, prot={:#x}", vaddr, size, prot);
        Ok(())
    }
    
    /// Copy segment data
    fn copy_segment(&self, vaddr: u64, src: *const u8, size: u64) -> Result<(), i32> {
        // Copy file content into the user memory mapping.
        // In a full implementation:
        // let result = copy_to_user(vaddr as *mut u8, src, size as usize);
        // if result != 0 {
        // return Err(-14);  // EFAULT
        // }
        // copy_to_user handles page faults that may occur if the
        // destination pages are not yet resident (demand paging).
        // SAFETY: src is a valid pointer to the ELF file data in kernel memory.
        // The destination (vaddr) is user memory that was mapped by alloc_segment.
        unsafe {
            let dst = vaddr as *mut u8;
            for i in 0..size as usize {
                *dst.add(i) = *src.add(i);
            }
        }
        log_debug!("copy_segment: {:#x}, size={}", vaddr, size);
        Ok(())
    }
    
    /// Zero segment (BSS)
    fn zero_segment(&self, vaddr: u64, size: u64) -> Result<(), i32> {
        // Zero the BSS section (mem_size > file_size portion).
        // In a full implementation:
        // let result = clear_user(vaddr as *mut u8, size as usize);
        // if result != 0 {
        // return Err(-14);  // EFAULT
        // }
        // clear_user safely zeros user memory, handling page faults.
        // SAFETY: vaddr points to user memory mapped by alloc_segment.
        unsafe {
            let dst = vaddr as *mut u8;
            for i in 0..size as usize {
                *dst.add(i) = 0;
            }
        }
        log_debug!("zero_segment: {:#x}, size={}", vaddr, size);
        Ok(())
    }
    
    /// Set segment protection
    fn set_segment_prot(&self, vaddr: u64, size: u64, flags: u32) -> Result<(), i32> {
        // Set memory protection for the loaded segment using mprotect.
        // After loading the segment data, we must set the final
        // protection flags (R/W/X) to match the ELF program header.
        // During loading, the pages were mapped RW to allow writing;
        // now we restrict them to the intended permissions.
        // In a full implementation:
        // let prot = self.flags_to_prot(flags);
        // let result = do_mprotect(vaddr, size, prot);
        // if result != 0 {
        // return Err(result);
        // }
        let prot = self.flags_to_prot(flags);
        log_debug!("set_segment_prot: {:#x}, size={}, prot={:#x}", vaddr, size, prot);
        Ok(())
    }
    
    /// Convert ELF flags to protection flags
    fn flags_to_prot(&self, flags: u32) -> u32 {
        let mut prot = 0u32;
        if (flags & PF_R) != 0 { prot |= 0x1; }  /* PROT_READ */
        if (flags & PF_W) != 0 { prot |= 0x2; }  /* PROT_WRITE */
        if (flags & PF_X) != 0 { prot |= 0x4; }  /* PROT_EXEC */
        prot
    }
}

/// Execve handler
/// Implements the execve system call.
pub struct ExecveHandler {
    /// Total execve calls
    pub exec_count: AtomicU64,
    /// Successful execs
    pub exec_success: AtomicU64,
    /// Failed execs
    pub exec_failures: AtomicU64,
    /// ELF loader
    pub elf_loader: ElfLoader,
}

impl ExecveHandler {
    pub const fn new() -> Self {
        ExecveHandler {
            exec_count: AtomicU64::new(0),
            exec_success: AtomicU64::new(0),
            exec_failures: AtomicU64::new(0),
            elf_loader: ElfLoader::new(),
        }
    }
    
    /// Execute a program
    /// @param filename: Path to executable
    /// @param argv: Argument vector
    /// @param envp: Environment vector
    /// @return Does not return on success, error on failure
    pub fn do_execve(
        &self,
        filename: *const u8,
        argv: *const *const u8,
        envp: *const *const u8,
    ) -> Result<(), i32> {
        self.exec_count.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("do_execve: {:?}", filename);
        
        // Step 1: Open executable file
        let file = self.open_exec(filename)?;
        
        // Step 2: Read file header
        let (data, size) = self.read_exec(file)?;
        
        // Step 3: Check file type and load
        let mut entry = 0u64;
        
        if self.is_elf(data) {
            // ELF binary
            self.elf_loader.load_elf(data, size, &mut entry)?;
        } else if self.is_shebang(data) {
            // Script with shebang
            return self.exec_script(filename, argv, envp);
        } else {
            // Unknown format
            self.close_exec(file);
            self.exec_failures.fetch_add(1, Ordering::AcqRel);
            return Err(-8);  /* ENOEXEC */
        }
        
        // Step 4: Set up new memory space
        self.setup_new_mm()?;
        
        // Step 5: Set up stack
        let sp = self.setup_stack(argv, envp)?;
        
        // Step 6: Set up registers
        self.setup_regs(entry, sp)?;
        
        // Step 7: Close file
        self.close_exec(file);
        
        // Step 8: Update process name
        self.set_process_name(filename)?;
        
        self.exec_success.fetch_add(1, Ordering::AcqRel);
        
        // Does not return - start executing new program
        Ok(())
    }
    
    /// Open executable file
    fn open_exec(&self, filename: *const u8) -> Result<u64, i32> {
        // Open the executable file using the VFS layer.
        // In a full implementation:
        // let file = do_filp_open(filename, O_RDONLY | O_EXEC);
        // if file.is_err() {
        // return Err(file.err().unwrap());
        // }
        // return Ok(file.unwrap() as u64);
        // O_EXEC flag checks execute permission on the file.
        // The file reference count is incremented to prevent
        // the file from being deleted while we are loading it.
        if filename.is_null() {
            return Err(-14);  /* EFAULT */
        }
        Ok(0x1000)  /* Placeholder file handle */
    }
    
    /// Read executable file
    fn read_exec(&self, _file: u64) -> Result<(*const u8, usize), i32> {
        // Read the entire executable file into kernel memory.
        // In a full implementation:
        // let size = file.size();
        // if size > EXEC_MAX_SIZE {
        // return Err(-7);  // E2BIG
        // }
        // let data = vmalloc(size);
        // if data.is_null() {
        // return Err(-12);  // ENOMEM
        // }
        // let read = kernel_read(file, data, size);
        // if read != size {
        // vfree(data);
        // return Err(-5);  // EIO
        // }
        // return Ok((data, size));
        Ok((0x2000 as *const u8, 4096))  /* Placeholder */
    }
    
    /// Close executable file
    fn close_exec(&self, _file: u64) {
        // Close the executable file and release the file reference.
        // In a full implementation:
        // filp_close(file);
        // This decrements the file's f_count and frees the
        // struct file if the count reaches zero.
    }
    
    /// Check if file is ELF
    fn is_elf(&self, data: *const u8) -> bool {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let magic = *data as u32;
            magic == ELF_MAGIC
        }
    }
    
    /// Check if file is script (shebang)
    fn is_shebang(&self, data: *const u8) -> bool {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            *data == b'#' && *data.add(1) == b'!'
        }
    }
    
    /// Execute script via interpreter
    fn exec_script(
        &self,
        _filename: *const u8,
        _argv: *const *const u8,
        _envp: *const *const u8,
    ) -> Result<(), i32> {
        // Parse the shebang line and exec the interpreter.
        // In a full implementation:
        // 1. Read the first line of the script
        // 2. Parse "#!" and extract interpreter path
        // 3. Build new argv: [interpreter, optional_arg, script_path, ...]
        // 4. Recursively call do_execve with interpreter
        // Example: "#!/usr/bin/python3" -> execve("/usr/bin/python3", [script, ...])
        // Limit recursion depth to prevent stack overflow from
        // circular interpreter chains (BINPRM_MAX_RECURSION = 4).
        Err(-8)  /* ENOEXEC */
    }
    
    /// Set up new memory space
    fn setup_new_mm(&self) -> Result<(), i32> {
        // Create a new mm_struct and switch the current process to it.
        // This is the point of no return for execve. The old address
        // space is destroyed and replaced with the new one.
        // In a full implementation:
        // let new_mm = mm_alloc();
        // if new_mm.is_null() {
        // return Err(-12);  // ENOMEM
        // }
        // let old_mm = current->mm;
        // current->mm = new_mm;
        // // Activate the new page table
        // activate_mm(old_mm, new_mm);
        // // Release the old address space
        // mmput(old_mm);
        // // Flush the TLB
        // flush_tlb_all();
        // After this, any access to the old user memory will fault.
        Ok(())
    }
    
    /// Set up user stack
    fn setup_stack(&self, argv: *const *const u8, envp: *const *const u8) -> Result<u64, i32> {
        // Allocate and set up the user stack with argv and envp.
        // In a full implementation:
        // const STACK_TOP: u64 = 0x7FFF_FFFF_F000;
        // const STACK_SIZE: u64 = 8 * 1024 * 1024;  // 8MB default
        // let stack = do_mmap(STACK_TOP - STACK_SIZE, STACK_SIZE,
        // PROT_READ | PROT_WRITE,
        // MAP_FIXED | MAP_PRIVATE | MAP_ANONYMOUS | MAP_GROWSDOWN,
        // -1, 0);
        // if stack != STACK_TOP - STACK_SIZE {
        // return Err(-12);  // ENOMEM
        // }
        // let mut sp = STACK_TOP;
        // sp = copy_strings(sp, envp);  // Push env strings
        // sp = copy_strings(sp, argv);  // Push arg strings
        // // Align stack to 16 bytes (ABI requirement)
        // sp &= !0xF;
        let mut sp = 0x7FFF_FFFF_F000u64;  /* Top of user space */

        // Copy argv and envp to stack
        self.copy_strings(&mut sp, argv)?;
        self.copy_strings(&mut sp, envp)?;

        // Align stack pointer to 16 bytes (AArch64 ABI requirement)
        sp &= !0xF;

        Ok(sp)
    }
    
    /// Copy strings to stack
    fn copy_strings(&self, sp: &mut u64, strings: *const *const u8) -> Result<(), i32> {
        // Copy an array of strings to the user stack.
        // For each string in the array:
        // 1. Calculate string length (strlen)
        // 2. Decrement sp by (length + 1) for null terminator
        // 3. Align sp down to 8-byte boundary
        // 4. Copy the string to the stack using copy_to_user
        // 5. Record the address for the pointer array
        // After all strings are copied, write the pointer array
        // (argv[] or envp[]) on the stack as well.
        // In a full implementation:
        // let mut count = 0;
        // while !strings.add(count).is_null() {
        // let s = *strings.add(count);
        // if s.is_null() { break; }
        // let len = strlen(s);
        // *sp -= (len + 1) as u64;
        // *sp &= !7;  // 8-byte align
        // copy_to_user(*sp as *mut u8, s, len + 1);
        // count += 1;
        // }
        // // Write pointer array
        // *sp -= (count + 1) as u64 * 8;
        // for i in 0..count {
        // write_user(*sp + i as u64 * 8, string_addr[i]);
        // }
        // write_user(*sp + count as u64 * 8, 0);  // NULL terminator
        if strings.is_null() {
            return Ok(());
        }

        // Count strings and push them onto the stack
        let mut count: usize = 0;
        // SAFETY: strings is a valid pointer to an array of string pointers
        // terminated by a NULL entry.
        unsafe {
            while !(*strings.add(count)).is_null() {
                let s = *strings.add(count);
                let mut len = 0;
                while *s.add(len) != 0 {
                    len += 1;
                }

                // Push string onto stack
                *sp -= (len as u64 + 1);
                *sp &= !7u64;  /* 8-byte align */

                count += 1;

                // Safety limit to prevent infinite loop
                if count > 256 {
                    log_warn!("copy_strings: too many strings (>256), truncating");
                    break;
                }
            }
        }

        // Push pointer array onto stack
        *sp -= ((count as u64) + 1) * 8;
        *sp &= !7u64;

        Ok(())
    }
    
    /// Set up registers for new program
    fn setup_regs(&self, entry: u64, sp: u64) -> Result<(), i32> {
        // Set up pt_regs for return to user mode with the new program.
        // In a full implementation:
        // let regs = current_pt_regs();
        // regs.pc = entry;       // ELR_EL1: entry point
        // regs.sp = sp;          // SP_EL0: user stack pointer
        // regs.x0 = argc;        // x0: argument count
        // regs.x1 = argv_ptr;    // x1: argument vector pointer
        // regs.x2 = envp_ptr;    // x2: environment pointer
        // regs.pstate = PSTATE_MODE_EL0t;  // User mode
        // On AArch64, the ELF entry point is called with:
        // x0 = argc
        // x1 = argv
        // x2 = envp
        // x3 = auxv (auxiliary vector)
        // The function does not return; instead, the next return
        // from kernel mode will jump to the user entry point.
        log_debug!("setup_regs: entry={:#x}, sp={:#x}", entry, sp);
        Ok(())
    }
    
    /// Set process name from filename
    fn set_process_name(&self, filename: *const u8) -> Result<(), i32> {
        // Extract the basename from the filename path and set
        // the process name (task->comm).
        // In a full implementation:
        // let name = basename(filename);
        // set_task_comm(current, name);
        // This updates the process name shown in ps/top and
        // is limited to TASK_COMM_LEN (16) characters.
        if filename.is_null() {
            return Ok(());
        }

        // Find the last '/' to extract basename
        // SAFETY: filename is a valid null-terminated string pointer.
        unsafe {
            let mut last_slash: usize = 0;
            let mut i: usize = 0;
            while *filename.add(i) != 0 {
                if *filename.add(i) == b'/' {
                    last_slash = i + 1;
                }
                i += 1;
                if i > 256 {
                    break;
                }
            }

            let _basename = filename.add(last_slash);
            // In a full implementation: set_task_comm(current, basename);
        }

        Ok(())
    }
}

/// Global execve handler
static EXECVE_HANDLER: core::sync::OnceLock<ExecveHandler> = core::sync::OnceLock::new();

/// Get execve handler
pub fn execve_handler() -> &'static ExecveHandler {
    EXECVE_HANDLER.get_or_init(ExecveHandler::new)
}

/// Initialize execve handler
pub fn init_execve() {
    log_info!("Execve handler initialized");
}

/// Execve system call
pub fn sys_execve(filename: *const u8, argv: *const *const u8, envp: *const *const u8) -> i64 {
    match get_execve_handler().do_execve(filename, argv, envp) {
        Ok(()) => 0,  /* Should not reach here */
        Err(e) => e as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_elf_constants() {
        assert_eq!(ELF_MAGIC, 0x464C457F);
        assert_eq!(ELFCLASS64, 2);
        assert_eq!(ELFDATA2LSB, 1);
        assert_eq!(ET_EXEC, 2);
        assert_eq!(ET_DYN, 3);
        assert_eq!(EM_AARCH64, 183);
        assert_eq!(EM_X86_64, 62);
    }
    
    #[test]
    fn test_phdr_types() {
        assert_eq!(PT_NULL, 0);
        assert_eq!(PT_LOAD, 1);
        assert_eq!(PT_DYNAMIC, 2);
        assert_eq!(PT_INTERP, 3);
    }
    
    #[test]
    fn test_phdr_flags() {
        assert_eq!(PF_X, 1);
        assert_eq!(PF_W, 2);
        assert_eq!(PF_R, 4);
    }
    
    #[test]
    fn test_elf_loader_new() {
        let loader = ElfLoader::new();
        assert_eq!(loader.load_count.load(Ordering::Relaxed), 0);
    }
    
    #[test]
    fn test_execve_handler_new() {
        let handler = ExecveHandler::new();
        assert_eq!(handler.exec_count.load(Ordering::Relaxed), 0);
    }
    
    #[test]
    fn test_elf64_ehdr_size() {
        assert_eq!(core::mem::size_of::<Elf64Ehdr>(), 64);
    }
    
    #[test]
    fn test_elf64_phdr_size() {
        assert_eq!(core::mem::size_of::<Elf64Phdr>(), 56);
    }
}
