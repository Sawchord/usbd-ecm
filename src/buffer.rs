//! This module implements the transmit and receive buffers.
//! There is the possibility to use synchronization mechanisms, to facilitate
//! implementation of `smoltcp`.

use crate::{EP_PKG_USIZE, ETH_FRAME_SIZE};

#[derive(Debug, Clone)]
pub struct RxBufInner {
   buf: [u8; ETH_FRAME_SIZE],
   idx: usize,
   complete: bool,
}

impl RxBufInner {
   /// Returns `true`, if a frame has been received completely and
   /// can be passed on to the next layer
   pub fn frame_complete(&self) -> bool {
      self.complete
   }

   /// Resets the buffer into its initial state
   pub fn reset(&mut self) {
      self.idx = 0;
      self.complete = false;
   }

   /// If a frame is ready, it is returned.
   /// Reutrns `None` oterhwise
   pub fn try_get_frame(&self) -> Option<&[u8]> {
      match self.frame_complete() {
         false => None,
         true => {
            let idx = self.idx;
            Some(&self.buf[..idx])
         }
      }
   }

   /// Returns mutably the part of the buffer that
   /// is not written yet, such that a data packet can be copied into it
   /// NOTE: The buffer part left can be empty
   pub fn insert_packet(&mut self) -> &mut [u8] {
      let idx = self.idx;
      &mut self.buf[idx..]
   }

   /// After writing data using `insert`, the buffer needs to be advanced
   /// by the amount of data, that has been written
   pub fn advance(&mut self, num_bytes: usize) {
      // Advance the index into the reveive buffer
      self.idx += num_bytes;

      // If the received packet is short, the packet was
      // received completely
      if num_bytes < EP_PKG_USIZE {
         self.complete = true;
      }
   }
}

#[derive(Debug, Clone)]
pub struct TxBufInner {
   buf: [u8; ETH_FRAME_SIZE],
   idx: usize,
   len: usize,
}

impl TxBufInner {
   /// Returns `true`, if there is a frame in the process of being sent
   pub fn is_sending(&self) -> bool {
      self.len != 0
   }

   /// Resets the buffer into its initial state
   pub fn reset(&mut self) {
      self.idx = 0;
      self.len = 0;
   }

   /// Get the section of the frame to be sent next
   pub fn try_get_packet(&self) -> Option<&[u8]> {
      match self.is_sending() {
         false => None,
         true => {
            // Calculate the section of the frame that forms the next packet
            let bytes_to_send = EP_PKG_USIZE.min(self.len - self.idx);
            let idx_end = self.idx + bytes_to_send;

            Some(&self.buf[self.idx..idx_end])
         }
      }
   }

   /// Advance the buffer after reading data from it.
   /// If the advancement appens via a short packet, the frame has been sent
   /// completely and the buffer is rest
   pub fn advance(&mut self, num_bytes: usize) {
      // Advance the counter
      self.idx += num_bytes;

      if num_bytes < EP_PKG_USIZE {
         self.reset()
      }
   }

   /// Tries to mutably aqcuire the buffer in order to copy
   /// the data into it.
   /// Returns none, if there is already a frame in transit.
   pub fn try_send_frame(&mut self, len: usize) -> Option<&mut [u8]> {
      match self.is_sending() {
         true => None,
         false => {
            self.len = len;
            self.idx = 0;

            Some(&mut self.buf[..len])
         }
      }
   }
}

/// Structure holds and manages the receive side.
#[derive(Debug, Clone)]
pub struct RxBuf(RxBufInner);

impl RxBuf {
   pub fn new() -> Self {
      Self(RxBufInner {
         buf: [0; ETH_FRAME_SIZE],
         idx: 0,
         complete: false,
      })
   }

   pub fn lock_mut(&mut self) -> Option<&mut RxBufInner> {
      Some(&mut self.0)
   }
}
/// Stucture holds and manages the send side
#[derive(Debug, Clone)]
pub struct TxBuf(TxBufInner);

impl TxBuf {
   pub fn new() -> Self {
      Self(TxBufInner {
         buf: [0; ETH_FRAME_SIZE],
         idx: 0,
         len: 0,
      })
   }

   pub fn lock_mut(&mut self) -> Option<&mut TxBufInner> {
      Some(&mut self.0)
   }
}
