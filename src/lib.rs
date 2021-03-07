#![no_std]

use crate::{
    buffer::{RxBuf, TxBuf},
    ecm::CdcEcmClass,
};
use usb_device::{
    bus::{StringIndex, UsbBus, UsbBusAllocator},
    class::{ControlIn, ControlOut, UsbClass},
    descriptor::DescriptorWriter,
    endpoint::EndpointAddress,
    Result as UsbResult, UsbError,
};

pub(crate) mod buffer;
pub(crate) mod ecm;

#[cfg(feature = "smoltcp")]
pub(crate) mod lock;

#[cfg(feature = "smoltcp")]
pub(crate) mod smoltcp;

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
pub const ETH_FRAME_SIZE: usize = 1514;

/// The device class of this device.
pub const USB_CLASS_CDC: u8 = 0x02;

/// An implementation of [`UsbClass`]()
// TODO: Documentation
pub struct UsbEthernetDevice<'a, B: UsbBus> {
    ecm: CdcEcmClass<'a, B>,
    tx_buf: TxBuf,
    rx_buf: RxBuf,
}

impl<'a, B: UsbBus> UsbEthernetDevice<'a, B> {
    /// Create a new [`UsbEthernetDevice`]('UsbEthernetDevice').
    pub fn new(alloc: &'a UsbBusAllocator<B>, mac_addr: &[u8; 6]) -> Self {
        Self {
            ecm: CdcEcmClass::new(alloc, mac_addr),
            tx_buf: TxBuf::new(),
            rx_buf: RxBuf::new(),
        }
    }

    /// Check, wether an ethernet frame is ready to be received
    pub fn frame_ready(&mut self) -> bool {
        match self.rx_buf.lock_mut() {
            None => false,
            Some(buf) => buf.frame_complete(),
        }
    }

    /// Tries to receive an ethernet frame.
    ///
    /// If a frame is ready, the closure will be executed, which allows to copy out the ethernet frame.
    ///
    /// # Returns
    /// - the length of the frame, if a frame was received
    /// - `None`: otherwise
    pub fn try_receive_frame<F>(&mut self, f: F) -> Option<usize>
    where
        F: FnOnce(&[u8]),
    {
        #[allow(unused_mut)]
        let mut buf = match self.rx_buf.lock_mut() {
            None => return None,
            Some(buf) => buf,
        };

        match buf.try_get_frame() {
            None => None,
            Some(frame) => {
                let len = frame.len();
                f(frame);

                // Reset the buffer after reading it
                buf.reset();
                Some(len)
            }
        }
    }

    /// Attempts to receive data into rx_buf
    fn try_recv(&mut self) {
        #[allow(unused_mut)]
        let mut buf = match self.rx_buf.lock_mut() {
            None => return,
            Some(buf) => buf,
        };

        // Do not receive if there is an ethernet packet waiting
        // The pipe will stall until the ethernet packet gets processed.
        if buf.frame_complete() {
            return;
        }

        // Read a packet from the host
        match self.ecm.get_read_ep().read(buf.insert_packet()) {
            Ok(bytes_read) => buf.advance(bytes_read),
            // This can only be triggered by a a host ingoring our boundaries
            Err(UsbError::BufferOverflow) => {
                log::warn!("received more data than fits in one ethernet packet, dropping packet");
                buf.reset();
            }
            // If busy, try again later
            // FIXME: Should be possible to trigger this, remove?
            Err(UsbError::WouldBlock) => log::warn!("would block should not be able to happen"),
            Err(err) => {
                log::error!("unexpected usb error: {:?}", err);
                //self.reset();
            }
        }
    }

    /// Tries to send an ethernet frame
    ///
    /// If the device is ready to send a frame, the closure is executed to allow copying in the bytes.
    ///
    /// # Returns
    /// - `true`, if the packet was sent
    /// - `false` otherwise
    pub fn try_send_frame<F>(&mut self, len: usize, f: F) -> bool
    where
        F: FnOnce(&mut [u8]),
    {
        // If length to big, we simply return
        if len >= ETH_FRAME_SIZE {
            return false;
        }

        #[allow(unused_mut)]
        let result = match self.tx_buf.lock_mut() {
            None => false,
            Some(mut buf) => match buf.try_send_frame(len) {
                None => false,
                Some(buf) => {
                    f(buf);
                    true
                }
            },
        };

        // Trigger sending the first packet
        self.try_send();
        result
    }

    /// Attempts to write data out to the host from tx_buf
    fn try_send(&mut self) {
        #[allow(unused_mut)]
        let mut buf = match self.tx_buf.lock_mut() {
            None => return,
            Some(buf) => buf,
        };

        // Skip if there is no data to send
        if !buf.is_sending() {
            return;
        }

        // Retreive the packet
        let pkg = match buf.try_get_packet() {
            None => return,
            Some(pkg) => pkg,
        };

        // Send the packet to the host
        match self.ecm.get_write_ep().write(pkg) {
            Ok(bytes_written) if pkg.len() == bytes_written => buf.advance(bytes_written),
            Ok(bytes_written) => {
                log::error!("wrote {} bytes, expected {}", bytes_written, pkg.len());
                //self.reset();
            }
            Err(UsbError::WouldBlock) => log::warn!("would block should not be able to happen"),
            Err(err) => {
                log::error!("received unexpected error {:?}", err);
                //self.reset();
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

    fn endpoint_in_complete(&mut self, addr: EndpointAddress) {
        if addr == self.ecm.get_write_ep().address() {
            self.try_send();
        }
    }

    fn reset(&mut self) {
        // TODO: How to do this threadsafe?
        //self.tx_idx = 0;
        //self.tx_len = 0;
        //self.rx_idx = 0;
        //self.rx_complete = false;
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
