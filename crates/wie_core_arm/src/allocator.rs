use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_base::util::{read_generic, round_up, write_generic};

use crate::core::{ArmCore, HEAP_BASE};

const HEAP_SIZE: u32 = 0x100000;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct AllocationHeader {
    data: u32,
}

impl AllocationHeader {
    pub fn new(size: u32, in_use: bool) -> Self {
        Self {
            data: size | (in_use as u32) << 31,
        }
    }

    pub fn size(&self) -> u32 {
        self.data & 0x7FFFFFFF
    }

    pub fn in_use(&self) -> bool {
        self.data & 0x80000000 != 0
    }
}

pub struct Allocator {}

impl Allocator {
    pub fn init(core: &mut ArmCore) -> anyhow::Result<(u32, u32)> {
        core.map(HEAP_BASE, HEAP_SIZE)?;

        let header = AllocationHeader::new(HEAP_SIZE, false);

        write_generic(core, HEAP_BASE, header)?;

        Ok((HEAP_BASE, HEAP_SIZE))
    }

    pub fn alloc(core: &mut ArmCore, size: u32) -> anyhow::Result<u32> {
        let alloc_size = round_up(size as usize + size_of::<AllocationHeader>(), 4) as u32;

        let address = Self::find_address(core, alloc_size).ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;

        let previous_header: AllocationHeader = read_generic(core, address)?;

        let header = AllocationHeader::new(alloc_size, true);
        write_generic(core, address, header)?;

        // write next
        if previous_header.size() > alloc_size {
            let next_header = AllocationHeader::new(previous_header.size() - alloc_size, false);
            write_generic(core, address + alloc_size, next_header)?;
        }

        log::trace!("Allocated {:#x} bytes at {:#x}", size, address + size_of::<AllocationHeader>() as u32);

        Ok(address + size_of::<AllocationHeader>() as u32)
    }

    pub fn free(core: &mut ArmCore, address: u32) -> anyhow::Result<()> {
        let base_address = address - size_of::<AllocationHeader>() as u32;

        log::trace!("Freeing {:#x}", base_address);

        let header: AllocationHeader = read_generic(core, base_address)?;
        assert!(header.in_use());

        let header = AllocationHeader::new(header.size(), false);
        write_generic(core, base_address, header)?;

        Ok(())
    }

    pub fn get_total_memory() -> anyhow::Result<i32> {
        Ok(HEAP_SIZE as _)
    }

    fn find_address(core: &ArmCore, request_size: u32) -> Option<u32> {
        let mut cursor = HEAP_BASE;
        loop {
            let header: AllocationHeader = read_generic(core, cursor).ok()?;
            if !header.in_use() && header.size() >= request_size {
                return Some(cursor);
            } else {
                cursor += header.size();
            }

            if cursor >= HEAP_BASE + HEAP_SIZE {
                break;
            }
        }

        None
    }
}
