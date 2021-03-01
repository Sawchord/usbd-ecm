#![no_std]

pub(crate) mod ecm;

// We support both USB 1.1 packets with a size of 64 bytes
// as well as USB 2.0 packets with a size of 512 bytes.
// It is hardware dependent, wether the larger size is actually supported.
#[allow(dead_code)]
#[cfg(not(feature = "large_pkgs"))]
const EP_PKG_SIZE: u16 = 64;

#[allow(dead_code)]
#[cfg(feature = "large_pkgs")]
const EP_PKG_SIZE: u16 = 512;

/// The device class of this device.
pub const USB_CLASS_CDC: u8 = 0x02;
