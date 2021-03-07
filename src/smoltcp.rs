use crate::{
   buffer::{RxBufInner, TxBufInner},
   lock::LockHandle,
   UsbEthernetDevice,
};
use core::convert::TryInto;
use smoltcp::{
   phy::{Device, RxToken, TxToken},
   wire::EthernetAddress,
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

impl<'a> SmolUsb<'a> {}
