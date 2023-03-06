use serde::Deserialize;
use serde::Serialize;
use tracing::error;
use utils::snowflake::SNOWFLAKE;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum FrameType {
    Command,
    Data,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct UDPFrame {
    pub id: String,
    pub version: u8,
    pub frame_type: FrameType,
    pub length: u16,
    pub order: u8,
    pub order_count: u8,
    pub data: Vec<u8>,
}

impl UDPFrame {
    pub fn new(data: Vec<u8>) -> Self {
        let length = data.len() as u16;
        UDPFrame {
            id: SNOWFLAKE.lock().unwrap().generate().to_string(),
            version: 1u8,
            frame_type: FrameType::Data,
            length,
            order: 0,
            order_count: 0,
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
        UDPFrame {
            id: SNOWFLAKE.lock().unwrap().generate().to_string(),
            version: 1u8,
            frame_type: FrameType::Data,
            length,
            order: 0,
            order_count: 0,
            data,
        }
    }

    fn new_from_frame_bytes_order(frame: &Self, data: Vec<u8>, order: u8, order_count: u8) -> Self {
        let length = data.len() as u16;
        UDPFrame {
            id: frame.id.clone(),
            version: frame.version,
            frame_type: frame.frame_type.clone(),
            length,
            order,
            order_count,
            data,
        }
    }
}

impl UDPFrame {
    pub fn split_frame(&self) -> Vec<UDPFrame> {
        let mut result = vec![];
        if self.data.len() > 1000 {
            let chunks = self.data.chunks(1000);
            let len = chunks.len();
            for (index, chunk) in chunks.enumerate() {
                let frame = UDPFrame::new_from_frame_bytes_order(
                    self,
                    chunk.to_vec(),
                    index as u8,
                    len as u8,
                );
                result.push(frame);
            }
        } else {
            result.push(self.clone());
        }
        result
    }

    pub fn merge_frames(mut frames: Vec<UDPFrame>) -> Self {
        let mut data = vec![];
        frames.sort();
        for frame in frames {
            data.extend_from_slice(&frame.data);
        }
        UDPFrame::new_from(data)
    }
}

impl UDPFrame {
    pub fn from_vec(bytes: Vec<u8>) -> Option<Self> {
        match postcard::from_bytes(&bytes) {
            Ok(frame) => Some(frame),
            Err(e) => {
                error!("Prase frame error: {:?}, data length: {:?}", e, bytes.len());
                None
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }
}

impl From<UDPFrame> for Vec<u8> {
    fn from(frame: UDPFrame) -> Self {
        frame.to_bytes()
    }
}

impl Ord for UDPFrame {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialOrd for UDPFrame {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_frame() {
        let frame = UDPFrame::new_from("hello world".to_string());
        let bytes = frame.to_bytes();
        let frame = UDPFrame::from_vec(bytes);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.data, "hello world".as_bytes());
    }

    #[test]
    fn test_split_and_merge_frame() {
        let frame = UDPFrame::new_from("hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world".to_string());
        let frames = frame.split_frame();
        let frame = UDPFrame::merge_frames(frames);
        assert_eq!(frame.data, "hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world".as_bytes());
    }
}
