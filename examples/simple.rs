use std::mem;

use atomic_pool::{pool, Box};

#[derive(Debug)]
struct Packet(u32);

pool!(PacketPool: [Packet; 4]);

fn main() {
    let box1 = Box::<PacketPool>::new(Packet(1));
    println!("allocated: {:?}", box1);
    assert_eq!(box1.as_ref().map(|b| b.0), Some(1));

    let box2 = Box::<PacketPool>::new(Packet(2));
    println!("allocated: {:?}", box2);

    let box3 = Box::<PacketPool>::new(Packet(3));
    println!("allocated: {:?}", box3);

    let box4 = Box::<PacketPool>::new(Packet(4));
    println!("allocated: {:?}", box4);

    let box5 = Box::<PacketPool>::new(Packet(5));
    println!("5th allocation fails because the pool is full: {:?}", box5);

    println!("dropping another allocation...");
    mem::drop(box1);

    let box5 = Box::<PacketPool>::new(Packet(5));
    println!("5th allocation now works: {:?}", box5);
}
