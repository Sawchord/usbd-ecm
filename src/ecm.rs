use crate::{EP_PKG_SIZE, USB_CLASS_CDC};
use usb_device::{
    bus::{InterfaceNumber, StringIndex, UsbBus, UsbBusAllocator},
    class::{ControlIn, ControlOut, UsbClass},
    control::{Recipient, Request, RequestType},
    descriptor::DescriptorWriter,
    endpoint::{EndpointIn, EndpointOut},
    Result as UsbResult,
};

const USB_CLASS_CDC_DATA: u8 = 0x0a;
const CDC_SUBCLASS_ECM: u8 = 0x06;
const CDC_PROTOCOL_NONE: u8 = 0x00;

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_HEADER: u8 = 0x00;
const CDC_TYPE_UNION: u8 = 0x06;
const ETHERNET_FUNCTIONAL_DESCRIPTOR: u8 = 0x0F;

// CDC Class requests
const REQ_SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
const REQ_GET_ENCAPSULATED_COMMAND: u8 = 0x01;

// CDC ECM Class requests Section 6.2 in CDC ECM spec
const SET_ETHERNET_MULTICAST_FILTERS: u8 = 0x40;
const SET_ETHERNET_POWER_MANAGEMENT_PATTERN_FILTER: u8 = 0x41;
const GET_ETHERNET_POWER_MANAGEMENT_PATTERN_FILTER: u8 = 0x42;
const SET_ETHERNET_PACKET_FILTER: u8 = 0x43;
const GET_ETHERNET_STATISTICS: u8 = 0x44;

// CDC ECM Class notification codes, Section 6.3 in CDC ECM spec
// NOT IMPLEMENTED
//const NETWORK_CONNECTION: u8 = 0x00;
//const RESPONSE_AVAILABLE: u8 = 0x01;
//const CONNECTION_SPEED_CHANGE: u8 = 0x2A;

pub struct CdcEcmClass<'a, B: UsbBus> {
    comm_if: InterfaceNumber,
    comm_ep: EndpointIn<'a, B>,
    data_if: InterfaceNumber,
    read_ep: EndpointOut<'a, B>,
    write_ep: EndpointIn<'a, B>,

    mac_string_index: StringIndex,
    mac_string: [u8; 12],
}

// TODO: Implement Debug

impl<'a, B: UsbBus> CdcEcmClass<'a, B> {
    /// Create e new [`CdcEcmClass`](CdcEcmClass)
    pub fn new(alloc: &'a UsbBusAllocator<B>, mac_addr: &[u8; 6]) -> Self {
        // Generat the mac string as a bytes sequence
        let mut mac_str = [0; 12];
        hex::encode_to_slice(mac_addr, &mut mac_str).unwrap();

        Self {
            comm_if: alloc.interface(),
            comm_ep: alloc.interrupt(8, 255),
            data_if: alloc.interface(),
            read_ep: alloc.bulk(EP_PKG_SIZE),
            write_ep: alloc.bulk(EP_PKG_SIZE),

            mac_string_index: alloc.string(),
            mac_string: mac_str,
        }
    }

    /// Checks, whether this request was directed to this class
    fn is_for_me(&self, req: &Request) -> bool {
        req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.index == u8::from(self.comm_if) as u16
    }

    /// Get the in endpoint
    pub fn get_write_ep(&self) -> &'a EndpointIn<B> {
        &self.write_ep
    }

    /// Get the out endpoint
    pub fn get_read_ep(&self) -> &'a EndpointOut<B> {
        &self.read_ep
    }
}

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
                // bNumberPowerFilters - No Wakeup feature
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

        Ok(())
    }

    fn get_string(&self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        // If the mac address is requested, we return it as a str
        if index == self.mac_string_index {
            // We know that this is valid because of the output of the hex library
            Some(unsafe { core::str::from_utf8_unchecked(&self.mac_string) })
        } else {
            None
        }
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();
        if !self.is_for_me(req) {
            return;
        }

        match req.request {
            REQ_GET_ENCAPSULATED_COMMAND => {
                log::error!("encapsulated commands are not supported");
                xfer.reject().ok();
            }
            GET_ETHERNET_POWER_MANAGEMENT_PATTERN_FILTER => {
                log::error!("power management not supported");
                xfer.reject().ok();
            }
            GET_ETHERNET_STATISTICS => {
                log::error!("statistics not supported");
                xfer.reject().ok();
            }
            _ => {
                log::error!("rejecting unkown IN request code {}", req.request);
                xfer.reject().ok();
            }
        }
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();
        if !self.is_for_me(req) {
            return;
        }

        match req.request {
            REQ_SEND_ENCAPSULATED_COMMAND => {
                log::error!("encapsulated commands are not supported");
                xfer.reject().ok();
            }
            SET_ETHERNET_MULTICAST_FILTERS => {
                // TODO: Implement this mandatory feature
                log::error!("ethernet multicast filters are not supported");
                xfer.reject().ok();
            }
            SET_ETHERNET_PACKET_FILTER => {
                log::error!("ethernet packet filters are not supported");
                xfer.reject().ok();
            }
            SET_ETHERNET_POWER_MANAGEMENT_PATTERN_FILTER => {
                log::error!("power management not supported");
            }
            _ => {
                log::error!("rejecting unkown OUT request code {}", req.request);
                xfer.reject().ok();
            }
        }
    }
}
