mod ws;
use socketcan::{CANFrame, CANSocket};

fn main() {
    let d:[u8; 8] = [1, 2,3, 4,5,6,7,8];
    let f = CANFrame::new(0x123, &d, false, false).unwrap();
    let s = CANSocket::open("can0").unwrap();
    s.write_frame(&f);


    println!("Hello, world!");
}
