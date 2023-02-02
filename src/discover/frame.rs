use serde::Deserialize;
use serde::Serialize;
use tracing::error;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum FrameType {
    Notify,
    Response,
    Request,
    ResponseAck,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Frame {
    pub id: String,
    pub version: u8,
    pub frame_type: u8,
    pub length: u16,
    pub order: u8,
    pub order_count: u8,
    pub data: Vec<u8>,
}

impl Frame {
    pub fn new(data: Vec<u8>) -> Self {
        let length = data.len() as u16;
        Frame {
            id: uuid::Uuid::new_v4().to_string(),
            version: 1u8,
            frame_type: 1u8,
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
        Frame {
            id: uuid::Uuid::new_v4().to_string(),
            version: 1u8,
            frame_type: 1u8,
            length,
            order: 0,
            order_count: 0,
            data,
        }
    }

    fn new_from_frame_bytes_order(frame: Self, data: Vec<u8>, order: u8, order_count: u8) -> Self {
        let length = data.len() as u16;
        Frame {
            id: frame.id,
            version: frame.version,
            frame_type: frame.frame_type,
            length,
            order,
            order_count,
            data,
        }
    }
}

impl Frame {
    pub fn split_frame(&self) -> Vec<Frame> {
        let mut result = vec![];
        if self.data.len() > 1000 {
            let chunks = self.data.chunks(1000);
            let len = chunks.len();
            for (index, chunk) in chunks.enumerate() {
                let frame = Frame::new_from_frame_bytes_order(
                    self.clone(),
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

    pub fn merge_frames(mut frames: Vec<Frame>) -> Self {
        let mut data = vec![];
        frames.sort();
        for frame in frames {
            data.extend_from_slice(&frame.data);
        }
        Frame::new_from(data)
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl Ord for Frame {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialOrd for Frame {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Frame {
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

impl From<Frame> for Vec<u8> {
    fn from(frame: Frame) -> Self {
        frame.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_frame() {
        let frame = Frame::new_from("hello world".to_string());
        let bytes = frame.to_bytes();
        let frame = Frame::from_vec(bytes);
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.data(), "hello world".as_bytes());
    }

    #[test]
    fn test_split_frame() {
        let frame = Frame::new_from("hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world".to_string());
        let frames = frame.split_frame();
        let frame = Frame::merge_frames(frames);
        assert_eq!(frame.data(), "hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world".as_bytes());
    }
}
