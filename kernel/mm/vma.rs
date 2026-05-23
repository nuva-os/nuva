/*
* Nuva OS - Kernel - Mm
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

use core::ptr;

/** VMA merge policy for controlling when adjacent VMAs are coalesced.
 *
 * - Immediate: Merge VMAs as soon as they become adjacent (default).
 * - Delayed: Defer merging until an explicit merge request or
 *   memory pressure event. Reduces VMA tree rotations for
 *   workloads with frequent mmap/munmap cycles.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmaMergePolicy {
    /** Merge adjacent VMAs immediately on insertion */
    Immediate,
    /** Defer merging until explicitly requested */
    Delayed,
}

/** Default VMA merge policy */
pub const DEFAULT_MERGE_POLICY: VmaMergePolicy = VmaMergePolicy::Immediate;

/// VMA Flag
pub mod vm_flags {
    pub const VM_NONE: u64 = 0x00000000;
    pub const VM_READ: u64 = 0x00000001;
    pub const VM_WRITE: u64 = 0x00000002;
    pub const VM_EXEC: u64 = 0x00000004;
    pub const VM_SHARED: u64 = 0x00000008;
    pub const VM_MAYREAD: u64 = 0x00000010;
    pub const VM_MAYWRITE: u64 = 0x00000020;
    pub const VM_MAYEXEC: u64 = 0x00000040;
    pub const VM_MAYSHARE: u64 = 0x00000080;
    pub const VM_GROWSDOWN: u64 = 0x00000100;
    pub const VM_UFFD: u64 = 0x00000200;
    pub const VM_PFNMAP: u64 = 0x00000400;
    pub const VM_DENYWRITE: u64 = 0x00000800;
    pub const VM_UFFD_WP: u64 = 0x00001000;
    pub const VM_LOCKED: u64 = 0x00002000;
    pub const VM_IO: u64 = 0x00004000;
    pub const VM_SEQ_READ: u64 = 0x00008000;
    pub const VM_RAND_READ: u64 = 0x00010000;
    pub const VM_DONTCOPY: u64 = 0x00020000;
    pub const VM_DONTEXPAND: u64 = 0x00040000;
    pub const VM_LOCKONFAULT: u64 = 0x00080000;
    pub const VM_ACCOUNT: u64 = 0x00100000;
    pub const VM_NORESERVE: u64 = 0x00200000;
    pub const VM_HUGETLB: u64 = 0x00400000;
    pub const VM_SYNC: u64 = 0x00800000;
    pub const VM_ARCH_1: u64 = 0x01000000;
    pub const VM_WIPEONFORK: u64 = 0x02000000;
    pub const VM_DONTDUMP: u64 = 0x04000000;
    pub const VM_MERGEABLE: u64 = 0x08000000;
}

/// VMA struct
#[repr(C)]
pub struct Vma {
    /// startbeginimaginarysimulatedAddress
    pub vm_start: u64,
    /// EndimaginarysimulatedAddress
    pub vm_end: u64,
    /// Next VMA
    pub vm_next: *mut Vma,
    /// prefixaitem VMA
    pub vm_prev: *mut Vma,
    /// Flag
    pub vm_flags: u64,
    /// pageprotected
    pub vm_page_prot: u64,
    /// placebelongProcess
    pub vm_mm: u64,
    /// FileMap
    pub vm_file: u64,
    /// Offset
    pub vm_pgoff: u64,
}

impl Vma {
    pub const fn new() -> Self {
        Vma {
            vm_start: 0,
            vm_end: 0,
            vm_next: ptr::null_mut(),
            vm_prev: ptr::null_mut(),
            vm_flags: 0,
            vm_page_prot: 0,
            vm_mm: 0,
            vm_file: 0,
            vm_pgoff: 0,
        }
    }

    /// GetSize
    pub fn size(&self) -> u64 {
        self.vm_end - self.vm_start
    }

    /// CheckAddressifin VMA inside
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.vm_start && addr < self.vm_end
    }

    /// Checkifcanread
    pub fn is_readable(&self) -> bool {
        (self.vm_flags & vm_flags::VM_READ) != 0
    }

    /// Checkifcanwrite
    pub fn is_writable(&self) -> bool {
        (self.vm_flags & vm_flags::VM_WRITE) != 0
    }

    /// Checkifcanexecute
    pub fn is_executable(&self) -> bool {
        (self.vm_flags & vm_flags::VM_EXEC) != 0
    }
}

/** Red-black tree node for VMA lookup.
 *
 * The max_end field stores the maximum vm_end value in the
 * subtree rooted at this node. This augmentation enables
 * O(log n) search for the VMA containing a given address
 * without traversing the entire tree.
 */
#[repr(C)]
pub struct VmaRbNode {
    /** Parent node pointer (color stored in LSB) */
    pub rb_parent: u64,
    /** Left child node pointer */
    pub rb_left: u64,
    /** Right child node pointer */
    pub rb_right: u64,
    /** Maximum vm_end in the subtree rooted at this node.
     *
     * This field must be updated after any tree rotation
     * or node insertion/deletion. It enables the augmented
     * red-black tree search optimization where subtrees
     * whose max_end <= addr can be pruned entirely.
     */
    pub max_end: u64,
    /** Pointer back to the owning VMA */
    pub vma: *mut Vma,
}

impl VmaRbNode {
    /** Create a new red-black tree node for the given VMA */
    pub const fn new(vma: *mut Vma) -> Self {
        VmaRbNode {
            rb_parent: 0,
            rb_left: 0,
            rb_right: 0,
            max_end: 0,
            vma,
        }
    }

    /** Update max_end from children and own VMA.
     *
     * After tree modifications, this must be called
     * bottom-up to maintain the augmentation invariant.
     */
    pub fn update_max_end(&mut self, own_end: u64) {
        self.max_end = own_end;
        // TODO: Incorporate children's max_end when tree ops are implemented
    }
}

/// MemoryDescriptor
pub struct MmStruct {
    /// Codeparagraphstartbegin
    pub start_code: u64,
    /// CodeparagraphEnd
    pub end_code: u64,
    /// Dataparagraphstartbegin
    pub start_data: u64,
    /// DataparagraphEnd
    pub end_data: u64,
    /// Heapstartbegin
    pub start_brk: u64,
    /// HeapEnd
    pub brk: u64,
    /// Stackstartbegin
    pub start_stack: u64,
    /// Parameterstartbegin
    pub arg_start: u64,
    /// ParameterEnd
    pub arg_end: u64,
    /// Ringenvironmentstartbegin
    pub env_start: u64,
    /// RingenvironmentEnd
    pub env_end: u64,

    /// VMA linkform
    pub mmap: *mut Vma,
    /// VMA count
    pub map_count: u32,

    /// totalimaginarysimulatedMemorySize
    pub total_vm: u64,
    /// LockfixedMemorySize
    pub locked_vm: u64,
    /// DataSize
    pub data_vm: u64,
    /// executeSize
    pub exec_vm: u64,
    /// StackSize
    pub stack_vm: u64,

    /// Page Tablebaseaddress
    pub pgd: u64,

    /// referenceCount
    pub mm_users: u32,
    /// referenceCount
    pub mm_count: u32,
}

impl MmStruct {
    pub const fn new() -> Self {
        MmStruct {
            start_code: 0,
            end_code: 0,
            start_data: 0,
            end_data: 0,
            start_brk: 0,
            brk: 0,
            start_stack: 0,
            arg_start: 0,
            arg_end: 0,
            env_start: 0,
            env_end: 0,
            mmap: ptr::null_mut(),
            map_count: 0,
            total_vm: 0,
            locked_vm: 0,
            data_vm: 0,
            exec_vm: 0,
            stack_vm: 0,
            pgd: 0,
            mm_users: 0,
            mm_count: 0,
        }
    }

    /// FindPackageAddress VMA
    pub fn find_vma(&self, addr: u64) -> Option<&Vma> {
        let mut vma = self.mmap;

        while !vma.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*vma).vm_start <= addr && addr < (*vma).vm_end {
                    return Some(&*vma);
                }
                vma = (*vma).vm_next;
            }
        }

        None
    }

    /// FindorCreate VMA
    pub fn find_vma_or_create(&mut self, addr: u64, len: u64, flags: u64) -> Option<&mut Vma> {
        // Findfinite VMA
        if let Some(vma) = self.find_vma(addr) {
            // SAFETY: unsafe block required for low-level memory or hardware access
            return Some(unsafe { &mut *(vma as *const Vma as *mut Vma) });
        }

        // Create new VMA
        // Implementation of actual VMA creation
        let vma = Box::leak(Vma {
            vm_start: start,
            vm_end: end,
            vm_flags: flags,
            vm_page_prot: prot,
            vm_ops: None,
            vm_private_data: core::ptr::null_mut(),
            vm_prev: core::ptr::null_mut(),
            vm_next: core::ptr::null_mut(),
            vm_rb: core::ptr::null_mut(),
        });

        Some(vma)
    }

    /// Insert VMA
    pub fn insert_vma(&mut self, vma: *mut Vma) {
        if vma.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // FindInsertPosition
            let mut prev = ptr::null_mut();
            let mut next = self.mmap;

            while !next.is_null() && (*next).vm_start < (*vma).vm_start {
                prev = next;
                next = (*next).vm_next;
            }

            // Insert
            (*vma).vm_prev = prev;
            (*vma).vm_next = next;

            if !prev.is_null() {
                (*prev).vm_next = vma;
            } else {
                self.mmap = vma;
            }

            if !next.is_null() {
                (*next).vm_prev = vma;
            }
        }

        self.map_count += 1;
    }

    /// Delete VMA
    pub fn erase_vma(&mut self, vma: *mut Vma) {
        if vma.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if !(*vma).vm_prev.is_null() {
                (*(*vma).vm_prev).vm_next = (*vma).vm_next;
            } else {
                self.mmap = (*vma).vm_next;
            }

            if !(*vma).vm_next.is_null() {
                (*(*vma).vm_next).vm_prev = (*vma).vm_prev;
            }
        }

        self.map_count -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vma() {
        let mut vma = Vma::new();
        vma.vm_start = 0x1000;
        vma.vm_end = 0x2000;

        assert_eq!(vma.size(), 0x1000);
        assert!(vma.contains(0x1500));
        assert!(!vma.contains(0x2000));
    }
}
