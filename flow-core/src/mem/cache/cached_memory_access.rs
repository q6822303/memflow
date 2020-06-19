use super::{page_cache::PageCache, page_cache::PageValidity, CacheValidator};
use crate::architecture::Architecture;
use crate::error::Result;
use crate::iter::PageChunks;
use crate::mem::phys_mem::{PhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use crate::types::{Length, PageType};

use bumpalo::Bump;

pub struct CachedMemoryAccess<'a, T: PhysicalMemory, Q: CacheValidator> {
    mem: T,
    cache: PageCache<'a, Q>,
    arena: Bump,
}

impl<'a, T: PhysicalMemory, Q: CacheValidator> CachedMemoryAccess<'a, T, Q> {
    pub fn with(mem: T, cache: PageCache<'a, Q>) -> Self {
        Self {
            mem,
            cache,
            arena: Bump::new(),
        }
    }

    pub fn builder() -> CachedMemoryAccessBuilder<T, Q> {
        CachedMemoryAccessBuilder::default()
    }
}

// forward PhysicalMemory trait fncs
impl<'a, T: PhysicalMemory, Q: CacheValidator> PhysicalMemory for CachedMemoryAccess<'a, T, Q> {
    fn phys_read_iter<'b, PI: PhysicalReadIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.cache.validator.update_validity();
        self.arena.reset();
        self.cache.cached_read(&mut self.mem, iter, &self.arena)
    }

    fn phys_write_iter<'b, PI: PhysicalWriteIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.cache.validator.update_validity();

        let cache = &mut self.cache;
        let mem = &mut self.mem;

        let iter = iter.inspect(move |(addr, data)| {
            if cache.is_cached_page_type(addr.page_type()) {
                for (paddr, data_chunk) in data.page_chunks(addr.address(), cache.page_size()) {
                    let mut cached_page = cache.cached_page_mut(paddr, false);
                    if let PageValidity::Valid(buf) = &mut cached_page.validity {
                        // write-back into still valid cache pages
                        let start = (paddr - cached_page.address).as_usize();
                        buf[start..(start + data_chunk.len())].copy_from_slice(data_chunk);
                    }

                    cache.put_entry(cached_page);
                }
            }
        });

        mem.phys_write_iter(iter)
    }
}

pub struct CachedMemoryAccessBuilder<T, Q> {
    mem: Option<T>,
    validator: Option<Q>,
    page_size: Option<Length>,
    cache_size: Length,
    page_type_mask: PageType,
}

impl<T: PhysicalMemory, Q: CacheValidator> Default for CachedMemoryAccessBuilder<T, Q> {
    fn default() -> Self {
        Self {
            mem: None,
            validator: None,
            page_size: None,
            cache_size: Length::from_mb(2),
            page_type_mask: PageType::PAGE_TABLE | PageType::READ_ONLY,
        }
    }
}

impl<'a, T: PhysicalMemory, Q: CacheValidator> CachedMemoryAccessBuilder<T, Q> {
    pub fn build(self) -> Result<CachedMemoryAccess<'a, T, Q>> {
        Ok(CachedMemoryAccess::with(
            self.mem.ok_or("mem must be initialized")?,
            PageCache::with_page_size(
                self.page_size.ok_or("page_size must be initialized")?,
                self.cache_size,
                self.page_type_mask,
                self.validator.ok_or("validator must be initialized")?,
            ),
        ))
    }

    pub fn mem(mut self, mem: T) -> Self {
        self.mem = Some(mem);
        self
    }

    pub fn validator(mut self, validator: Q) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn page_size(mut self, page_size: Length) -> Self {
        self.page_size = Some(page_size);
        self
    }

    pub fn cache_size(mut self, cache_size: Length) -> Self {
        self.cache_size = cache_size;
        self
    }

    pub fn arch(mut self, arch: Architecture) -> Self {
        self.page_size = Some(arch.page_size());
        self
    }

    pub fn page_type_mask(mut self, page_type_mask: PageType) -> Self {
        self.page_type_mask = page_type_mask;
        self
    }
}
