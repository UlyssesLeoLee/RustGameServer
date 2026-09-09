// SmartSocket binary frame parser/builder (per zsyz_client_core/.../GameTcpClient.h)
// All integers big-endian (network byte order)
// Frame: | len:32 BE | cmd:16 BE | payload |
//   len = 2 + payload.len  (cmd 2 bytes + payload)

use bytes::{Bytes, BytesMut};

#[allow(dead_code)]
pub const MAX_FRAME_SIZE: usize = 128 * 1024;

pub struct Frame {
    pub cmd: u16,
    pub payload: Bytes,
}

impl Frame {
    #[allow(dead_code)]
    pub fn parse(buf: &[u8]) -> Result<Frame, FrameError> {
        if buf.len() < 6 {
            return Err(FrameError::TooShort);
        }
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let cmd = u16::from_be_bytes([buf[4], buf[5]]);
        let payload_len = len.checked_sub(2).ok_or(FrameError::InvalidLen)?;
        if buf.len() < 6 + payload_len {
            return Err(FrameError::TooShort);
        }
        let payload = Bytes::copy_from_slice(&buf[6..6 + payload_len]);
        Ok(Frame { cmd, payload })
    }

    #[allow(dead_code)]
    pub fn encode(&self) -> Bytes {
        let mut out = BytesMut::with_capacity(6 + self.payload.len());
        let len = (2 + self.payload.len()) as u32;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(&self.cmd.to_be_bytes());
        out.extend_from_slice(&self.payload);
        out.freeze()
    }
}

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum FrameError {
    #[error("frame too short")]
    TooShort,
    #[error("invalid length")]
    InvalidLen,
    #[error("frame too large: {0} > {1}")]
    TooLarge(usize, usize),
}

// === Big-endian primitive read/write on &[u8] / BytesMut ===
pub trait BeRead {
    fn read_u8(&mut self) -> u8;
    fn read_u16(&mut self) -> u16;
    fn read_u32(&mut self) -> u32;
    fn read_i16(&mut self) -> i16;
    fn read_string(&mut self) -> String;
}

pub trait BeWrite {
    fn write_u8(&mut self, v: u8);
    fn write_u16(&mut self, v: u16);
    fn write_u32(&mut self, v: u32);
    #[allow(dead_code)]
    fn write_i16(&mut self, v: i16);
    fn write_string(&mut self, s: &str);
}

impl BeRead for &[u8] {
    fn read_u8(&mut self) -> u8 {
        let v = self[0];
        *self = &self[1..];
        v
    }
    fn read_u16(&mut self) -> u16 {
        let v = u16::from_be_bytes([self[0], self[1]]);
        *self = &self[2..];
        v
    }
    fn read_u32(&mut self) -> u32 {
        let v = u32::from_be_bytes([self[0], self[1], self[2], self[3]]);
        *self = &self[4..];
        v
    }
    fn read_i16(&mut self) -> i16 {
        self.read_u16() as i16
    }
    fn read_string(&mut self) -> String {
        let len = self.read_u32() as usize;
        let bytes = &self[..len];
        *self = &self[len..];
        String::from_utf8_lossy(bytes).to_string()
    }
}

impl BeWrite for Vec<u8> {
    fn write_u8(&mut self, v: u8) { self.extend_from_slice(&v.to_be_bytes()); }
    fn write_u16(&mut self, v: u16) { self.extend_from_slice(&v.to_be_bytes()); }
    fn write_u32(&mut self, v: u32) { self.extend_from_slice(&v.to_be_bytes()); }
    fn write_i16(&mut self, v: i16) { self.write_u16(v as u16); }
    fn write_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_u32(bytes.len() as u32);
        self.extend_from_slice(bytes);
    }
}
