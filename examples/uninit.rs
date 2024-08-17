use atomic_pool::pool;

pool!(TestPool: [u32; 3]);

#[cfg(feature = "alloc_uninit")]
fn main() {
    use atomic_pool::Box;
    use std::mem;
    let mut buffer = unsafe { Box::<TestPool>::new_uninit() }.unwrap();
    println!("Allocated new buffer.");

    *buffer = 0xf00dbabeu32;

    let _buffer_2 = unsafe { Box::<TestPool>::new_uninit() }.unwrap();
    let _buffer_3 = unsafe { Box::<TestPool>::new_uninit() }.unwrap();

    mem::drop(buffer);
    println!("Dropped buffer.");

    let reallocated_buffer = unsafe { Box::<TestPool>::new_uninit() }.unwrap();
    println!(
        "Reallocated buffer, with contents: {:#x}",
        *reallocated_buffer
    );
}
#[cfg(not(feature = "alloc_uninit"))]
fn main() {}
