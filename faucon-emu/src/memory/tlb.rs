/// Flag bits for managing access to physical memory pages.
#[derive(Debug)]
#[repr(u8)]
pub enum PageFlag {
    /// Indicates that the page is mapped and complete and can be used.
    Usable = 1 << 0,
    /// Indicates that the page is mapped but code is still being uploaded.
    Busy = 1 << 1,
}

/// Potential TLB lookup errors.
///
/// These may occur when doing virtual <-> physical page translations.
#[derive(Debug)]
pub enum LookupError {
    /// A page fault that occurs when no TLB entries could be matched for a
    /// physical page.
    NoPageHits,
    /// A page fault that occurs when multiple TLB entries could be matched
    /// for a single physical page.
    MultiplePageHits,
}

/// The Falcon Translation Lookaside Buffer for mapping code pages in memory.
///
/// It consists of multiple [`TlbEntry`]s, each representing one physical page.
///
/// The valid virtual address range is defined as `0..(1 << (UC_CAPS2 >> 16 & 0xF)) * 0x100`
/// and whenever such an address is accessed, the TLB searches for a corresponding
/// entry. If there is more than one match or no match at all, it is considered an
/// error and a trap should be generated by the CPU.
///
/// [`TlbEntry`]: struct.TlbEntry.html
pub struct Tlb {
    /// The entries of the TLB, used for page lookup.
    entries: Vec<TlbEntry>,
}

impl Tlb {
    /// Creates a new instance of the TLB for virtual address translation.
    pub fn new() -> Self {
        Tlb {
            entries: vec![TlbEntry::new(); 0x80],
        }
    }

    /// Gets a mutable reference to the [`TlbEntry`] that corresponds to the given
    /// physical address.
    ///
    /// [`TlbEntry`]: struct.TlbEntry.html
    pub fn get_physical_entry(&mut self, address: u16) -> &mut TlbEntry {
        &mut self.entries[(address >> 8) as usize]
    }

    /// Translates a virtual address to a physical address.
    pub fn translate_addr(&self, address: u32) -> Result<u16, LookupError> {
        let (page_index, _) = self.lookup(address)?;
        let page_offset = (address & 0xFF) as u16;

        Ok(((page_index as u16) << 8) | page_offset)
    }

    /// Finds a [`TlbEntry`] that corresponds to the given virtual address
    /// and returns a reference to it.
    ///
    /// If a page fault occurs, a [`LookupError`] is returned.
    ///
    /// In case the page is found, a `(page_index, entry)` constellation is
    /// returned, where `page_index` denotes the physical page index and `entry`
    /// the [`TlbEntry`] that was found.
    ///
    /// [`TlbEntry`]: struct.TlbEntry.html
    /// [`LookupError`]: enum.LookupError.html
    pub fn lookup(&self, address: u32) -> Result<(u8, &TlbEntry), LookupError> {
        // Calculate the virtual page number to look up.
        let page_index = (address >> 8) as u16 & ((1 << 8) - 1);

        // Find all the valid entries that match the virtual page number.
        let mut entries = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_valid() && e.virtual_page_number == page_index)
            .map(|(i, e)| (i as u8, e))
            .collect::<Vec<_>>();

        // Count the hits and derive the appropriate result.
        if entries.len() == 1 {
            Ok(entries.pop().unwrap())
        } else if entries.len() == 0 {
            Err(LookupError::NoPageHits)
        } else {
            Err(LookupError::MultiplePageHits)
        }
    }

    /// Finds a [`TlbEntry`] that corresponds to the given virtual address
    /// and builds a result value in the following format:
    ///
    /// - Bits 0:7   - physical page index
    /// - Bits 8:23  - 0
    /// - Bits 24:26 - flags, ORed across all matches
    /// - Bit  30    - Set if multiple pages were hit
    /// - Bit  31    - Set if no pages were hit
    ///
    /// The above format is generated by VTLB operations on the hardware and
    /// thus, this method is used to emulate its behavior in instructions.
    ///
    /// [`TlbEntry`]: struct.TlbEntry.html
    pub fn lookup_raw(&self, address: u32) -> u32 {
        let mut physical_index = 0;
        let mut flags = 0;

        // Calculate the virtual page number to look up.
        let page_index = (address >> 8) as u16 & ((1 << 8) - 1);

        // Find all the valid entries that match the virtual page number.
        let entries = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_valid() && e.virtual_page_number == page_index)
            .map(|(i, e)| (i as u8, e))
            .collect::<Vec<_>>();

        // Extract the physical page index and the flags accordingly.
        for (page, entry) in entries.iter() {
            flags |= entry.flags;
            physical_index = *page;
        }

        // Build the result value.
        let mut result = (physical_index as u32) | (flags as u32) << 24;
        if entries.len() == 0 {
            result |= 0x80000000;
        } else if entries.len() > 1 {
            result |= 0x40000000;
        }

        result
    }

    /// Finds a [`TlbEntry`] that corresponds to the given virtual address
    /// and returns a mutable reference to it.
    ///
    /// If a page fault occurs, a [`LookupError`] is returned.
    ///
    /// In case the page is found, a `(page_index, entry)` constellation is
    /// returned, where `page_index` denotes the physical page index and `entry`
    /// the [`TlbEntry`] that was foud.
    ///
    /// [`TlbEntry`]: struct.TlbEntry.html
    /// [`LookupError`]: enum.LookupError.html
    pub fn lookup_mut(&mut self, address: u32) -> Result<(u8, &mut TlbEntry), LookupError> {
        // Calculate the virtual page number to look up.
        let page_index = (address >> 8) as u16 & ((1 << 8) - 1);

        // Find all the valid entries that match the virtual page number.
        let mut entries = self
            .entries
            .iter_mut()
            .enumerate()
            .filter(|(_, e)| e.is_valid() && e.virtual_page_number == page_index)
            .map(|(i, e)| (i as u8, e))
            .collect::<Vec<_>>();

        // Count the hits and derive the appropriate result.
        if entries.len() == 1 {
            Ok(entries.pop().unwrap())
        } else if entries.len() == 0 {
            Err(LookupError::NoPageHits)
        } else {
            Err(LookupError::MultiplePageHits)
        }
    }
}

/// An entry in the [`Tlb`] that represents a physical code page.
///
/// [`Tlb`]: struct.Tlb.html
#[derive(Clone, Copy, Debug)]
pub struct TlbEntry {
    /// The virtual page number corresponding to a physical page.
    pub virtual_page_number: u16,
    /// The status flag bits for a physical page.
    pub flags: u8,
}

impl TlbEntry {
    /// Creates a new entry for the TLB, marked as completely free
    /// and unmapped.
    pub fn new() -> Self {
        TlbEntry {
            virtual_page_number: 0,
            flags: 0,
        }
    }

    /// Maps the physical page corresponding to the TLB entry to the virtual page
    /// space the given address belongs to.
    ///
    /// NOTE: This sets [`PageFlag::Busy`]. It is within the caller's
    /// responsibility to change this after code has been uploaded.
    ///
    /// [`PageFlag::Busy`]: enum.PageFlag.html#variant.Busy
    pub fn map(&mut self, address: u32, _secret: bool) {
        self.virtual_page_number = (address >> 8) as u16 & ((1 << 8) - 1);
        self.set_flag(PageFlag::Busy, true);
    }

    /// Toggles a flag in the page settings based on the value of `set`.
    ///
    /// - `set = true` sets the given flag
    /// - `set = false` clears the given flag
    pub fn set_flag(&mut self, flag: PageFlag, set: bool) {
        if set {
            self.flags |= flag as u8;
        } else {
            self.flags &= !(flag as u8);
        }
    }

    /// Gets a flag from the page settings and indicates whether it is set.
    pub fn get_flag(&self, flag: PageFlag) -> bool {
        (self.flags & flag as u8) != 0
    }

    /// Checks if the entry is considered valid.
    pub fn is_valid(&self) -> bool {
        self.flags != 0
    }

    /// Indicates whether the physical page corresponding to the TLB entry
    /// is currently unmapped.
    pub fn is_free(&self) -> bool {
        self.virtual_page_number == 0 && self.flags == 0
    }

    /// Clears the TLB entry and frees it for remapping.
    ///
    /// NOTE: Pages containing secret code cannot be cleared.
    /// The page has to be re-uploaded with non-secret data first.
    pub fn clear(&mut self) {
        self.virtual_page_number = 0;
        self.flags = 0;
    }
}
