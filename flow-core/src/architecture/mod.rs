/*!
This module contains all architecture definitions currently
supported by memflow.

Each architecture is wrapped in the `Architecture` enum
and all function calls are dispatched into their own
architecture specific sub-modules.

Each architecture also has a `ByteOrder` assigned to it.
When reading/writing data from/to the target it is necessary
that memflow know the proper byte order of the target system.
*/

pub mod x64;
pub mod x86;
pub mod x86_pae;

use crate::error::{Error, Result};
use std::convert::TryFrom;

use crate::mem::AccessPhysicalMemory;
use crate::types::{Address, Length, PhysicalAddress};

/**
Identifies the byte order of a architecture
*/
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ByteOrder {
    /// little endianess
    LittleEndian,
    /// big endianess
    BigEndian,
}

/**
Describes the architecture to of a target.
The architecture will contain information about the pointer width,
byte order, page size and also how to translate virtual to physical memory.
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Architecture {
    /**
    An empty architecture with some sensible defaults and no virt_to_phys translation.
    This is usually most useful when running automated tests.
    */
    Null,
    /// x86_64 architecture.
    X64,
    /**
    x86 architecture with physical address extensions.
    See [here](https://en.wikipedia.org/wiki/Physical_Address_Extension) for more information on the subject.
    */
    X86Pae,
    /// x86 architecture.
    X86,
}

/**
Converts a `u8` value to an `Architecture`.
This is usually helpful when serializing / deserializing data in a safe way.

# Examples

```
use flow_core::architecture::Architecture;
use std::convert::TryFrom;

pub fn test() {
    let arch = Architecture::try_from(1).unwrap();
    assert_eq!(arch, Architecture::X64);
}
```
*/
impl TryFrom<u8> for Architecture {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Architecture::Null),
            1 => Ok(Architecture::X64),
            2 => Ok(Architecture::X86Pae),
            3 => Ok(Architecture::X86),
            _ => Err(Error::new("Invalid Architecture value")),
        }
    }
}

#[allow(dead_code)]
impl Architecture {
    /**
    Converts a `Architecture` to a corresponding `u8` value.
    This is usually helpful when serializing / deserializing data in a safe way.

    # Examples

    ```
    use flow_core::architecture::Architecture;

    pub fn test() {
        let arch = Architecture::X64;
        assert_eq!(arch.as_u8(), 1);
    }
    ```
    */
    pub fn as_u8(self) -> u8 {
        match self {
            Architecture::Null => 0,
            Architecture::X64 => 1,
            Architecture::X86Pae => 2,
            Architecture::X86 => 3,
        }
    }

    /**
    Returns the number of bits of a pointers width on a `Architecture`.
    Currently this will either return 64 or 32 depending on the pointer width of the target.
    This function is handy in cases where you only want to know the pointer width of the target\
    but you don't want to match against all architecture.

    # Examples

    ```
    use flow_core::architecture::Architecture;

    pub fn test() {
        let arch = Architecture::X86Pae;
        assert_eq!(arch.bits(), 32);
    }
    ```
    */
    pub fn bits(self) -> u8 {
        match self {
            Architecture::Null => x64::bits(),
            Architecture::X64 => x64::bits(),
            Architecture::X86Pae => x86_pae::bits(),
            Architecture::X86 => x86::bits(),
        }
    }

    /**
    Returns the byte order of an `Architecture`.
    This will either be `ByteOrder::LittleEndian` or `ByteOrder::BigEndian`.

    In most circumstances this will be `ByteOrder::LittleEndian` on all x86 and arm architectures.

    # Examples

    ```
    use flow_core::architecture::{Architecture, ByteOrder};

    pub fn test() {
        let arch = Architecture::X86;
        assert_eq!(arch.byte_order(), ByteOrder::LittleEndian);
    }
    ```
    */
    pub fn byte_order(self) -> ByteOrder {
        match self {
            Architecture::Null => x64::byte_order(),
            Architecture::X64 => x64::byte_order(),
            Architecture::X86Pae => x86_pae::byte_order(),
            Architecture::X86 => x86::byte_order(),
        }
    }

    /**
    Returns the smallest page size of an `Architecture`.

    In x86/64 and arm this will always return 4kb.

    # Examples

    ```
    use flow_core::architecture::{Architecture, ByteOrder};
    use flow_core::types::Length;

    pub fn test() {
        let arch = Architecture::X64;
        assert_eq!(arch.page_size(), Length::from_kb(4));
    }
    ```
    */
    pub fn page_size(self) -> Length {
        match self {
            Architecture::Null => x64::page_size(),
            Architecture::X64 => x64::page_size(),
            Architecture::X86Pae => x86_pae::page_size(),
            Architecture::X86 => x86::page_size(),
        }
    }

    /**
    Returns the `Length` of a pointers width on a `Architecture`.

    This function will return the pointer width as a `Length` value.
    See `Architecture::bits()` for more information.

    # Examples

    ```
    use flow_core::architecture::Architecture;
    use flow_core::types::Length;

    pub fn test() {
        let arch = Architecture::X86;
        assert_eq!(arch.len_addr(), Length::from(4));
    }
    ```
    */
    pub fn len_addr(self) -> Length {
        match self {
            Architecture::Null => x64::len_addr(),
            Architecture::X64 => x64::len_addr(),
            Architecture::X86Pae => x86_pae::len_addr(),
            Architecture::X86 => x86::len_addr(),
        }
    }

    /**
    This function will do a virtual to physical memory translation for the `Architecture`.

    TODO: add more info how virt_to_phys works

    # Examples

    TODO: add example
    */
    pub fn virt_to_phys<T: AccessPhysicalMemory>(
        self,
        mem: &mut T,
        dtb: Address,
        addr: Address,
    ) -> Result<PhysicalAddress> {
        let mut vec = Vec::new();
        self.virt_to_phys_iter(mem, dtb, Some(addr).into_iter(), &mut vec);
        vec.pop().unwrap()
    }

    pub fn virt_to_phys_iter<T: AccessPhysicalMemory, VI: Iterator<Item = Address>>(
        self,
        mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut Vec<Result<PhysicalAddress>>,
    ) -> () {
        match self {
            Architecture::Null => addrs.for_each(|addr| out.push(Ok(PhysicalAddress::from(addr)))),
            Architecture::X64 => x64::virt_to_phys_iter(mem, dtb, addrs, out),
            Architecture::X86Pae => x86_pae::virt_to_phys_iter(mem, dtb, addrs, out),
            Architecture::X86 => x86::virt_to_phys_iter(mem, dtb, addrs, out),
        }
    }
}
