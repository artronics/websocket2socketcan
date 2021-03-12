use serde_derive::{Deserialize, Serialize};
use serde_json::Result as SerdeResult;
use socketcan::{CANFrame, CANSocket, ConstructionError};

#[derive(Serialize, Deserialize)]
pub struct CanFrame {
    id: u32,
    data: [u8; 8],
    data_length: usize,
    is_remote: bool,
    is_error: bool,
}

impl CanFrame {
    pub fn new(id: u32, data: [u8; 8], data_length: usize, is_remote: bool, is_error: bool) -> Self {
        CanFrame {
            id,
            data,
            data_length,
            is_remote,
            is_error,
        }
    }
    pub fn from_linux_frame(f: CANFrame) -> Self {
        let mut data = [0; 8];
        for i in 0..f.data().len() {
            data[i] = f.data()[i];
        }
        CanFrame {
            id: f.id(),
            data,
            data_length: f.data().len(),
            is_error: f.is_error(),
            is_remote: f.is_rtr(),
        }
    }
    pub fn to_linux_frame(&self) -> CANFrame {
        CANFrame::new(self.id, &self.data, self.is_remote, self.is_error).unwrap()
    }
}

pub struct Can {
    socket: CANSocket
}

impl Can {
    pub fn new(can_if: &str) -> Self {
        let socket = CANSocket::open(can_if).unwrap();
        socket.set_nonblocking(true);
        Can {
            socket,
        }
    }

    pub fn write(&self, frame: CanFrame) {
        let linux_frame = frame.to_linux_frame();
        self.socket.write_frame(&linux_frame);
    }
}

