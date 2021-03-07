use crate::{EP_PKG_USIZE, ETH_FRAME_SIZE};

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
   /// is not written yet, such that data can be copied into it
   pub fn insert(&mut self) -> &mut [u8] {
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

/// Structure holds and manages the receive side.
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

//pub struct TxBufInner {}
