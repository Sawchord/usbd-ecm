//! This is a simple example, which tries to register a USB-ECM enables
//! device, which simply sends back all ethernet frames that it receives.
//!
//! This device should behave similar to the loopback device.

use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_ecm::{UsbEthernetDevice, ETH_FRAME_SIZE, USB_CLASS_CDC};
use usbip_device::UsbIpBus;

fn sleep() {
   std::thread::sleep(std::time::Duration::from_millis(1));
}

fn main() {
   pretty_env_logger::init();

   log::info!("initializing allocator");
   let bus_allocator = UsbBusAllocator::new(UsbIpBus::new());
   let mut usb_eth = UsbEthernetDevice::new(&bus_allocator, &[1, 2, 3, 4, 5, 6]);

   let mut usb_bus = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x16c0, 0x05e1))
      .manufacturer("Fake company")
      .product("Ethernet Loopback Device")
      .serial_number("TEST")
      .device_class(USB_CLASS_CDC)
      // NOTE: This is needed due to a bug.
      // I am not sure, wether the bug is in usb-device of usbip-device though.
      // The bug triggers a get_string, before the second one is transmitted.
      // This triggers the string being garbled which in turn triggers `cdc_ether`
      // to reject the device.
      .max_packet_size_0(64)
      .build();

   let mut data = [0; ETH_FRAME_SIZE];
   loop {
      // TODO: Only sleep, if there was no new data
      let _poll = usb_bus.poll(&mut [&mut usb_eth]);
      sleep();

      // Loop until we received some data
      let bytes_read = match usb_eth.try_receive_frame(|buf| data.copy_from_slice(buf)) {
         None => continue,
         Some(bytes_read) => bytes_read,
      };

      log::debug!("Received data: {:?}", &data[..bytes_read]);

      // Try to send back the data
      loop {
         // We need to continue polling the bus in order to drive it
         let _poll = usb_bus.poll(&mut [&mut usb_eth]);
         // FIXME: Needed?
         sleep();

         if usb_eth.try_send_frame(bytes_read, |buf| buf.copy_from_slice(&data[..bytes_read])) {
            break;
         };
      }
   }
}
