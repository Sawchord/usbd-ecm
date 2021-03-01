use crate::{EP_PKG_SIZE, USB_CLASS_CDC};
use usb_device::{
    bus::{InterfaceNumber, StringIndex, UsbBus, UsbBusAllocator},
    class::{ControlIn, ControlOut, UsbClass},
    descriptor::DescriptorWriter,
    endpoint::{EndpointIn, EndpointOut},
    Result as UsbResult,
};

const USB_CLASS_CDC_DATA: u8 = 0x0a;
const CDC_SUBCLASS_ECM: u8 = 0x06;
const CDC_PROTOCOL_NONE: u8 = 0x00;

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_HEADER: u8 = 0x00;
//const CDC_TYPE_CALL_MANAGEMENT: u8 = 0x01;
//const CDC_TYPE_ACM: u8 = 0x02;
const ETHERNET_FUNCTIONAL_DESCRIPTOR: u8 = 0x0F;
const CDC_TYPE_UNION: u8 = 0x06;

//const REQ_SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
//const REQ_GET_ENCAPSULATED_COMMAND: u8 = 0x01;

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

// TODO: Implement Debug

impl<B: UsbBus> UsbClass<B> for CdcEcmClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> UsbResult<()> {
        writer.iad(
            self.comm_if,
            2,
            USB_CLASS_CDC,
            CDC_SUBCLASS_ECM,
            CDC_PROTOCOL_NONE,
        )?;

        // Communications interface descriptor
        writer.interface(
            self.comm_if,
            USB_CLASS_CDC,
            CDC_SUBCLASS_ECM,
            CDC_PROTOCOL_NONE,
        )?;

        // Header functional descriptor
        writer.write(
            CS_INTERFACE,
            &[
                CDC_TYPE_HEADER, // bDescriptorSubtype
                0x10,
                0x01, // bcdCDC (1.10)
            ],
        )?;

        // Union functional descriptor
        writer.write(
            CS_INTERFACE,
            &[
                CDC_TYPE_UNION,      // bDescriptorSubtype
                self.comm_if.into(), // bControlInterface
                self.data_if.into(), // bSubordinateInterface
            ],
        )?;

        // Ethernet functional descriptor
        writer.write(
            CS_INTERFACE,
            &[
                ETHERNET_FUNCTIONAL_DESCRIPTOR,
                // String index of the MAC address
                self.mac_string_index.into(),
                // Ethernet Statstics capabilities (None)
                0x00,
                0x00,
                0x00,
                0x00,
                // wMaxSegmentSize - 1514 bytes
                0xEA,
                0x05,
                // wNumberMCFilters - No multicast filering
                0x00,
                0x00,
            ],
        )?;

        // Communications endpoint descriptor
        writer.endpoint(&self.comm_ep)?;

        // Data interface descriptor
        writer.interface(self.data_if, USB_CLASS_CDC_DATA, 0x00, 0x00)?;

        // Data IN endpoint descriptor
        writer.endpoint(&self.write_ep)?;

        // Data OUT endpoint descriptor
        writer.endpoint(&self.read_ep)?;

        todo!()
    }

    fn get_string(&self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        // If the mac address is requested, we return it as a str
        if index == self.mac_string_index {
            Some(core::str::from_utf8(&self.mac_string).unwrap())
        } else {
            None
        }
    }
}

impl<'a, B: UsbBus> CdcEcmClass<'a, B> {
    /// Create e new [`CdcEcmClass`](CdcEcmClass)
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
