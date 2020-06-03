pub fn morton2(x: usize, y: usize) -> usize {
    interleave_with_zeros(x) | (interleave_with_zeros(y) << 1)
}

pub fn unravel_point(index: usize) -> (usize, usize) {
    let x = unravel(index);
    let y = unravel(index >> 1);
    (x, y)
}

#[cfg(target_pointer_width = "64")]
pub fn interleave_with_zeros(mut n: usize) -> usize {
    n = (n ^ (n << 16)) & 0x0000_ffff_0000_ffff;
    n = (n ^ (n << 8)) & 0x00ff_00ff_00ff_00ff;
    n = (n ^ (n << 4)) & 0x0f0f_0f0f_0f0f_0f0f;
    n = (n ^ (n << 2)) & 0x3333_3333_3333_3333;
    (n ^ (n << 1)) & 0x5555_5555_5555_5555
}

#[cfg(target_pointer_width = "32")]
pub fn interleave_with_zeros(mut n: usize) -> usize {
    n = (n ^ (n << 8)) & 0x00ff_00ff; // (1)
    n = (n ^ (n << 4)) & 0x0f0f_0f0f; // (2)
    n = (n ^ (n << 2)) & 0x3333_3333; // (3)
    (n ^ (n << 1)) & 0x5555_5555 // (4)
}

#[cfg(target_pointer_width = "64")]
pub fn unravel(mut n: usize) -> usize {
    n &= 0x5555_5555;
    n |= n >> 1;
    n &= 0x3333_3333;
    n |= n >> 2;
    n &= 0x0f0f_0f0f;
    n |= n >> 4;
    n &= 0x00ff_00ff;
    n |= n >> 8;
    n & 0x0000_ffff
}

#[cfg(target_pointer_width = "32")]
pub fn unravel(mut n: usize) -> usize {
    n &= 0x5555_5555_5555_5555;
    n |= n >> 1;
    n &= 0x3333_3333_3333_3333;
    n |= n >> 2;
    n &= 0x0f0f_0f0f_0f0f_0f0f;
    n |= n >> 4;
    n &= 0x00ff_00ff_00ff_00ff;
    n |= n >> 8;
    n &= 0x0000_ffff_0000_ffff;
    n |= n >> 16;
    n & 0x0000_0000_ffff_ffff
}
