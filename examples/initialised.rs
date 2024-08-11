use atomic_pool::{pool, Box};
use std::mem;

pool!(TestPool: [u32; 3], [0u32; 3]);

fn main() {
    let mut buffer = unsafe { Box::<TestPool>::new_uninit() }.unwrap();
    println!("Allocated new buffer, with contents: {:#x}", *buffer);

    *buffer = 0xf00dbabeu32;

    let _buffer_2 = unsafe { Box::<TestPool>::new_uninit() }.unwrap();
    let _buffer_3 = unsafe { Box::<TestPool>::new_uninit() }.unwrap();

    mem::drop(buffer);
    println!("Dropped buffer.");

    let reallocated_buffer = unsafe { Box::<TestPool>::new_uninit() }.unwrap();
    println!(
        "Reallocated buffer, with contents: 0x{:#x}",
        *reallocated_buffer
    );
}
