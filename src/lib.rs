#![no_std]

use usb_device::{
    bus::{InterfaceNumber, StringIndex, UsbBus, UsbBusAllocator},
    endpoint::{EndpointIn, EndpointOut},
};

pub(crate) mod usbd;

// We support both USB 1.1 packets with a size of 64 bytes
// as well as USB 2.0 packets with a size of 512 bytes.
// It is hardware dependent, wether the larger size is actually supported.
#[cfg(not(feature = "large_pkgs"))]
const EP_PKG_SIZE: u16 = 64;
#[cfg(feature = "large_pkgs")]
const EP_PKG_SIZE: u16 = 512;

/// The device class of this device.
pub const USB_CLASS_CDC: u8 = 0x02;

pub struct CdcEcmClass<'a, B: UsbBus> {
    comm_if: InterfaceNumber,
    comm_ep: EndpointIn<'a, B>,
    data_if: InterfaceNumber,
    read_ep: EndpointOut<'a, B>,
    write_ep: EndpointIn<'a, B>,

    mac_string_index: StringIndex,
    mac_string: [u8; 12],
    // TODO: Add buffer stuff
}

impl<'a, B: UsbBus> CdcEcmClass<'a, B> {
    pub fn new(alloc: &'a UsbBusAllocator<B>, mac_addr: &[u8; 6]) -> Self {
        // Generat the mac string as a bytes sequence
        let mut mac_str = [0; 12];
        hex::decode_to_slice(mac_addr, &mut mac_str).unwrap();

        Self {
            comm_if: alloc.interface(),
            comm_ep: alloc.interrupt(64, 255),
            data_if: alloc.interface(),
            read_ep: alloc.bulk(EP_PKG_SIZE),
            write_ep: alloc.bulk(EP_PKG_SIZE),

            mac_string_index: alloc.string(),
            mac_string: mac_str,
        }
    }
}

// TODO: Implement Debug
