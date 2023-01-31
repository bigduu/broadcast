use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UdpFrame {
    version: u8,
    frame_type: u8,
    length: u16,
    pub data: Vec<u8>,
}

impl UdpFrame {
    pub fn new(data: Vec<u8>) -> Self {
        let length = data.len() as u16;
        UdpFrame {
            version: 1u8,
            frame_type: 1u8,
            length,
            data,
        }
    }

    pub fn new_from<T>(data: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        let data: Vec<u8> = data.into();
        //read u8
        let length = data.len() as u16;
        UdpFrame {
            version: 1u8,
            frame_type: 1u8,
            length,
            data,
        }
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        serde_json::from_slice(&bytes).unwrap()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

impl From<UdpFrame> for Vec<u8> {
    fn from(frame: UdpFrame) -> Self {
        frame.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_frame() {
        let frame = UdpFrame::new_from("hello world".to_string());
        let bytes = frame.to_bytes();
        let frame = UdpFrame::from_vec(bytes);
        assert_eq!(frame.data, "hello world".as_bytes());
    }
}
