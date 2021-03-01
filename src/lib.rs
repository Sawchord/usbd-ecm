#![no_std]

use crate::ecm::CdcEcmClass;
use usb_device::{
    bus::{StringIndex, UsbBus, UsbBusAllocator},
    class::{ControlIn, ControlOut, UsbClass},
    descriptor::DescriptorWriter,
    endpoint::EndpointAddress,
    Result as UsbResult, UsbError,
};

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

/// EP_PKG_SIZE as USIZE
const EP_PKG_USIZE: usize = EP_PKG_SIZE as usize;

/// Length of an ethernet frame
const ETH_FRAME_SIZE: usize = 1514;

/// The device class of this device.
pub const USB_CLASS_CDC: u8 = 0x02;

/// An implementation of [`UsbClass`]()
// TODO
pub struct UsbEthernetDevice<'a, B: UsbBus> {
    ecm: CdcEcmClass<'a, B>,
    tx_buf: [u8; ETH_FRAME_SIZE],
    tx_idx: usize,
    tx_len: usize,
    rx_buf: [u8; ETH_FRAME_SIZE],
    rx_idx: usize,
    rx_complete: bool,
}

impl<'a, B: UsbBus> UsbEthernetDevice<'a, B> {
    /// Create a new [`UsbEthernetDevice`]('UsbEthernetDevice').
    pub fn new(alloc: &'a UsbBusAllocator<B>, mac_addr: &[u8; 6]) -> Self {
        Self {
            ecm: CdcEcmClass::new(alloc, mac_addr),
            tx_buf: [0; ETH_FRAME_SIZE],
            tx_idx: 0,
            tx_len: 0,
            rx_buf: [0; ETH_FRAME_SIZE],
            rx_idx: 0,
            rx_complete: false,
        }
    }

    /// Tries to receive data into rx_buffer
    fn try_recv(&mut self) {
        // Do not receive, if there is an ethernet packet waiting
        // The pipe will stall until the ethernet packet gets processed.
        if self.rx_complete {
            return;
        }

        let idx = self.rx_idx;
        match self.ecm.get_read_ep().read(&mut self.rx_buf[idx..]) {
            Ok(bytes_written) => {
                // Advance the index into the reveive buffer
                self.rx_idx += bytes_written;

                // If the received packet is short, the packet was
                // received completely
                if bytes_written < EP_PKG_USIZE {
                    self.rx_complete = true;
                }
            }
            // This can only be triggered by a bug on the host side
            Err(UsbError::BufferOverflow) => {
                log::warn!("received more data than fits in one ethernet packet, dropping packet");
                self.rx_idx = 0;
            }
            // If busy, try again later
            // FIXME: Should be possible to trigger this, remove?
            Err(UsbError::WouldBlock) => {
                log::warn!("would block should not be possible")
            }
            Err(err) => {
                log::error!("unexpected usb error: {:?}", err);
                self.reset();
            }
        }
    }
}

impl<B: UsbBus> UsbClass<B> for UsbEthernetDevice<'_, B> {
    fn endpoint_out(&mut self, addr: EndpointAddress) {
        if addr == self.ecm.get_read_ep().address() {
            self.try_recv();
        }
    }

    // TODO: Handling of the Endpoints

    fn reset(&mut self) {
        self.tx_idx = 0;
        self.tx_len = 0;
        self.rx_idx = 0;
        self.rx_complete = false;
    }

    // Pass through the control and setup calls
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> UsbResult<()> {
        self.ecm.get_configuration_descriptors(writer)
    }

    fn get_string(&self, index: StringIndex, lang_id: u16) -> Option<&str> {
        self.ecm.get_string(index, lang_id)
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        self.ecm.control_in(xfer);
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        self.ecm.control_out(xfer);
    }
}
