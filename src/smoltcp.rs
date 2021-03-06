use crate::{
   buffer::{RxBufInner, TxBufInner},
   lock::{Guard, LockHandle},
   UsbEthernetDevice,
};
use core::convert::TryInto;
use smoltcp::{
   phy::{Device, DeviceCapabilities, RxToken, TxToken},
   time::Instant,
   wire::EthernetAddress,
   Result as SmolResult,
};
use usb_device::bus::{UsbBus, UsbBusAllocator};

pub struct SmolUsb<'a> {
   tx_buf: LockHandle<'a, TxBufInner>,
   rx_buf: LockHandle<'a, RxBufInner>,
}

impl<'a, B> UsbEthernetDevice<'a, B>
where
   B: UsbBus,
{
   // TODO: Documentation
   pub fn with_ethernet(alloc: &'a UsbBusAllocator<B>, addr: &EthernetAddress) -> Self {
      Self::new(alloc, &addr.as_bytes().try_into().unwrap())
   }

   // TODO: Documetation
   pub fn get_smol<'b>(&'b self) -> SmolUsb<'b> {
      SmolUsb {
         tx_buf: self.tx_buf.get_handle(),
         rx_buf: self.rx_buf.get_handle(),
      }
   }
}

impl<'a> Device<'a> for SmolUsb<'a> {
   type TxToken = UsbTxToken<'a>;
   type RxToken = UsbRxToken<'a>;

   fn capabilities(&self) -> DeviceCapabilities {
      let mut cap = DeviceCapabilities::default();
      cap.max_transmission_unit = 1514;
      cap.max_burst_size = Some(1);

      cap
   }

   fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
      // Try to acquire both buffers
      // Only proceed, if there is a frame ready and also no output transmission
      match (self.rx_buf.try_lock(), self.tx_buf.try_lock()) {
         (Some(rx_buf), Some(tx_buf)) => match (rx_buf.frame_complete(), tx_buf.is_sending()) {
            (true, false) => Some((UsbRxToken(rx_buf), UsbTxToken(tx_buf))),
            _ => None,
         },
         _ => None,
      }
   }

   fn transmit(&'a mut self) -> Option<Self::TxToken> {
      // Try to acquire the tx buffer
      // Return early, if there is a sending in progress
      match self.tx_buf.try_lock() {
         Some(tx_buf) => match tx_buf.is_sending() {
            false => Some(UsbTxToken(tx_buf)),
            true => None,
         },
         None => None,
      }
   }
}

impl<'a> SmolUsb<'a> {}

pub struct UsbTxToken<'a>(Guard<'a, TxBufInner>);

impl<'a> TxToken for UsbTxToken<'a> {
   fn consume<R, F>(mut self, _timestamp: Instant, len: usize, f: F) -> SmolResult<R>
   where
      F: FnOnce(&mut [u8]) -> SmolResult<R>,
   {
      // We know that we are ready to send, because we checked
      // and have not released the lock since
      f(self.0.try_send_frame(len).unwrap())
   }
}

pub struct UsbRxToken<'a>(Guard<'a, RxBufInner>);

impl<'a> RxToken for UsbRxToken<'a> {
   fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> SmolResult<R>
   where
      F: FnOnce(&mut [u8]) -> SmolResult<R>,
   {
      // We know that we have a frame ready because we checked
      // and have not released the lock since.
      f(self.0.try_get_frame().unwrap())
   }
}
