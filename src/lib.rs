pub mod bit_stream;
mod vla;

use std::{
    cmp::min,
    fmt::{Debug, Display, Error, Formatter},
    vec::Vec,
};

pub use bit_stream::BitStream;
use vla::VLA;

type Bit = u8;

pub struct Keccak<'a> {
    state: &'a mut VLA<Bit>,
    state1: &'a mut VLA<Bit>,
    w: usize, //depth, or length of each lane
    l: u64,   //log base 2 of w
}

macro_rules! mut_ref_swap {
    ($first:expr, $second:expr, $T:ty) => {
        unsafe {
            let ptr1: *mut $T = ($first);
            $first = &mut *($second as *mut $T);
            $second = &mut *ptr1;
        }
    };
}

impl Keccak<'_> {
    /// 0<=x<=width, 0<=y<=height, 0<=z<=depth
    /// gets from state
    #[inline]
    fn get(&self, x: usize, y: usize, z: usize) -> Bit {
        return self.state.get(self.w * (5 * y + x) + z);
    }

    /// sets state1
    #[inline]
    fn set(&mut self, x: usize, y: usize, z: usize, val: Bit) {
        self.state1.set(self.w * (5 * y + x) + z, val);
    }

    #[inline]
    fn set_state(&mut self, x: usize, y: usize, z: usize, val: Bit) {
        self.state.set(self.w * (5 * y + x) + z, val);
    }

    #[inline]
    fn get_idx(&self, x: usize, y: usize, z: usize) -> usize {
        return self.w * (5 * y + x) + z;
    }

    pub fn get_lane(&self, x: usize, y: usize) -> Vec<Bit> {
        let mut temp = Vec::new();
        for z in 0..self.w {
            temp.push(self.get(x, y, z));
        }
        return temp;
    }

    pub fn depth(&self) -> usize {
        return self.w;
    }

    /// state arrays depth initialized to message length / 25.
    /// If the message is not evenly divisble by 25 then the message size will
    /// be rounded down to the nearest multiple of 25, if message contains more
    /// bits than state then some bits in message are unused
    pub fn new(message: &BitStream) -> Self {
        unsafe {
            let mut temp = Keccak {
                state: &mut *(Box::into_raw(Box::from(VLA::new((message.len() / 25) * 25)))),
                state1: &mut *(Box::into_raw(Box::from(VLA::new((message.len() / 25) * 25)))),
                w: message.len() / 25,
                l: 0,
            };
            temp.l = f64::log2(temp.w as f64) as u64; //w should equal 2^(integer)
            for idx in 0..min(message.len(), temp.w * 25) {
                temp.state.set(idx, message[idx]);
            }
            return temp;
        }
    }

    /// behaves like new, except the state depth is set to size,
    /// if message contains fewer bit then the state, the remaining bits are
    /// zero-initialized.
    pub fn new_sized(message: &BitStream, size: usize) -> Self {
        unsafe {
            let mut temp = Keccak {
                state: &mut *(Box::into_raw(Box::from(VLA::new(size * 25)))),
                state1: &mut *(Box::into_raw(Box::from(VLA::new(size * 25)))),
                w: size,
                l: 0,
            };
            temp.l = f64::log2(temp.w as f64) as u64;
            for idx in 0..min(message.len(), temp.w * 25) {
                temp.state.set(idx, message[idx]);
            }
            return temp;
        }
    }

    pub fn get_state(&self) -> BitStream {
        let mut tmp = BitStream::new(self.w * 25);
        for idx in 0..(25 * self.w) {
            tmp.set(idx, self.state.get(idx));
        }
        return tmp;
    }

    /// copies and returns the first 8 bytes of self.state
    pub fn copy_to_u64(&self) -> u64 {
        let mut temp: u64 = 0;
        for idx in 0..64 {
            temp |= (self.state.get(idx) as u64) << (63 - idx);
        }
        return temp;
    }

    fn column_parity<'a>(&'a self, x: usize, z: usize) -> Bit {
        return self.get(x, 0, z)
            ^ self.get(x, 1, z)
            ^ self.get(x, 2, z)
            ^ self.get(x, 3, z)
            ^ self.get(x, 4, z);
    }

    fn theta<'a>(&'a mut self) {
        for x in 0..(5 as i64) {
            for z in 0..(self.w as i64) {
                let d = self.column_parity(((x - 1) % 5).abs() as usize, z as usize)
                    ^ self.column_parity(
                        (x + 1) as usize % 5,
                        ((z - 1) % self.w as i64).abs() as usize,
                    );
                for y in 0..(5 as usize) {
                    self.set(
                        x as usize,
                        y,
                        z as usize,
                        self.get(x as usize, y, z as usize) ^ d,
                    );
                }
            }
        }
        mut_ref_swap!(self.state, self.state1, VLA<Bit>);
    }

    fn rho<'a>(&'a mut self) {
        for z in 0..self.w {
            self.set(0, 0, z, self.get(0, 0, z))
        }
        // could use lookup table to reduce calculations
        let (mut x, mut y) = (1, 0);
        for t in 0..23 {
            for z in 0..self.w {
                self.set(
                    x,
                    y,
                    z,
                    self.get(
                        x,
                        y,
                        (z as i64 - (t + 1) * (t + 2) / 2).abs() as usize % self.w,
                    ),
                );
            }
            let temp = y;
            y = (2 * x + 3 * y) % 5;
            x = temp;
        }
        mut_ref_swap!(self.state, self.state1, VLA<Bit>);
    }

    fn pi<'a>(&'a mut self) {
        for x in 0..(5 as usize) {
            for y in 0..(5 as usize) {
                for z in 0..self.w {
                    self.state1
                        .set(self.get_idx(x, y, z), self.get((x + 3 * y) % 5, x, z));
                }
            }
        }
        mut_ref_swap!(self.state, self.state1, VLA<Bit>);
    }

    fn chi<'a>(&'a mut self) {
        for x in 0..(5 as usize) {
            for y in 0..(5 as usize) {
                for z in 0..self.w {
                    self.set(
                        x,
                        y,
                        z,
                        self.get(x, y, z)
                            ^ ((self.get((x + 1) % 5, y, z) ^ 1) & self.get((x + 2) % 5, y, z)),
                    );
                }
            }
        }
        mut_ref_swap!(self.state, self.state1, VLA<Bit>);
    }

    fn rc(t: u64) -> Bit {
        if t % 255 == 0 {
            return 1;
        }
        let mut state: u32 = 1;
        for _ in 1..t % 255 + 1 {
            state <<= 1;
            let eighth: u32 = (state & 0b100000000) >> 8;
            state ^= eighth;
            state ^= eighth << 3;
            state ^= eighth << 4;
            state ^= eighth << 5;
            state &= 0xFF;
        }
        return (state as u8) & 1;
    }

    fn iota(&mut self, round: u64) {
        let mut rc = BitStream::new(self.w);
        for j in 0..self.l {
            rc.set(
                i64::pow(2, j as u32) as usize - 1,
                Keccak::rc(j as u64 + 7 * round),
            )
        }
        for z in 0..self.w {
            self.set_state(0, 0, z, self.get(0, 0, z) ^ rc.get(z));
        }
    }

    pub fn keccak(&mut self, num_rounds: u64) {
        for r in (12 + 2 * self.l - num_rounds as u64)..(12 + 2 * self.l) {
            self.theta();
            self.rho();
            self.pi();
            self.chi();
            self.iota(r);
        }
    }

    /// pads message so it may be split evenly into blocks,
    /// each block is xor'ed with the current state, and a round of keccak is done
    pub fn sponge_absorb(&mut self, message: &mut BitStream) {
        if message.len() % (self.w * 25) != 0 {
            let pad = (f64::ceil(message.len() as f64 / (self.w * 25) as f64) as u64
                * (self.w * 25) as u64)
                - message.len() as u64;
            let mut temp = Vec::<u8>::new();
            temp.resize(pad as usize, 0_u8);
            message.add_val(temp.as_slice());
        }
        let block: usize = message.len() / (self.depth() * 25);
        let mut count: usize = 0;
        while count < message.len() {
            for x in 0..5 as usize {
                for y in 0..5 as usize {
                    for z in 0..self.depth() {
                        self.set_state(
                            x,
                            y,
                            z,
                            self.get(x, y, z) ^ message.get(self.depth() * (5 * y + x) + z),
                        );
                    }
                }
            }
            self.keccak(12 + 2 * self.l);
            count += block;
        }
    }
}

impl Debug for Keccak<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for idx in 0..25 {
            match write!(
                f,
                "x:{} y:{}, {:?}\n",
                idx % 5,
                idx / 5,
                self.get_lane(idx % 5, idx / 5)
            ) {
                Err(err) => return Err(err),
                _ => continue,
            }
        }
        return Ok(());
    }
}

/// allows use of ToString
impl Display for Keccak<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        return Debug::fmt(self, f);
    }
}

impl Drop for Keccak<'_> {
    fn drop(&mut self) {
        unsafe {
            let ptr1: *mut VLA<Bit> = self.state;
            let ptr2: *mut VLA<Bit> = self.state1;
            std::ptr::drop_in_place(ptr1);
            std::ptr::drop_in_place(ptr2);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::bit_stream::BitStream;
    use crate::vla::VLA;
    use crate::Keccak;
    use std::mem::size_of;

    #[test]
    fn test_vla() {
        let mut a = VLA::<u64>::new(10);
        for x in 0..10 {
            assert_eq!(a.len(), 10);
            assert_eq!(a.get(x), 0);
            a.set(x, x as u64);
            assert_eq!(a.get(x), x as u64);
        }
    }

    #[test]
    #[should_panic]
    fn test_vla_panic_get() {
        let mut a = VLA::<u64>::new(5);
        a.get(5);
        a.set(5, 1);
    }

    #[test]
    fn test_bitstream_get() {
        let bits = BitStream::from_u64(&[0x5d]);
        assert_eq!(bits.len(), 64);
        assert_eq!(bits.get(0), 0);
        assert_eq!(bits.get(1), 0);
        assert_eq!(bits.get(59), 1);
        assert_eq!(bits.get(58), 0);
        assert_eq!(bits.get(63), 1);
        assert_eq!(bits.get(62), 0);
    }

    #[test]
    fn test_bitstream_from_u64() {
        let bits = BitStream::from_u64(&[0x1234cdef]);
        let mut result: i64 = 0;
        assert_eq!(bits.len(), 64);
        for idx in 0..bits.len() {
            result |= (bits.get(idx) as i64) << (63 - idx);
        }
        assert_eq!(result, 0x1234cdef);
    }

    #[test]
    fn test_bitstream_from_val() {
        let bits = BitStream::from_val(&[0x1234cdef_u32]);
        let mut result: i64 = 0;
        assert_eq!(bits.len(), 32);
        for idx in 0..bits.len() {
            result |= (bits.get(idx) as i64) << (31 - idx);
        }
        assert_eq!(result, 0x1234cdef);
    }

    #[test]
    fn test_bitstream_try_from() {
        let slice: [usize; 1] = [0xFFEEDDCCBBAA9988];
        let bits = BitStream::try_from_val(&slice);
        let mut result: usize = 0;
        assert_eq!(bits.len(), size_of::<usize>() * 8);
        for idx in 0..size_of::<usize>() * 8 {
            result |= (bits.get(idx) as usize) << (size_of::<usize>() * 8 - 1 - idx);
        }
        assert_eq!(result, 0xFFEEDDCCBBAA9988);
    }

    #[test]
    fn test_bitstream_try_from_u8() {
        let slice: [u8; 11] = [
            0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77, 0x66, 0x55,
        ];
        let bits = BitStream::try_from_val(&slice);
        let mut result1: u64 = 0;
        let mut result2: u64 = 0;
        assert_eq!(bits.len(), slice.len() * 8);
        for idx in 0..64 {
            result1 |= (bits.get(idx) as u64) << (63 - idx);
        }
        for idx in 64..slice.len() * 8 {
            result2 |= (bits.get(idx) as u64) << (127 - idx);
        }
        assert_eq!(result1, 0xFFEEDDCCBBAA9988);
        assert_eq!(result2, 0x7766550000000000);
    }

    #[test]
    fn test_bitstream_from_str() {
        let bits = BitStream::from_str("abcdefgh");
        assert_eq!(bits.len(), 64);
        let mut result: i64 = 0;
        for idx in 0..bits.len() {
            result |= (bits.get(idx) as i64) << 63 - idx;
        }
        assert_eq!(result, 0x6162636465666768);
    }

    #[test]
    fn test_keccak_new() {
        // create 200 bit, 5*5*8 state array
        let state = Keccak::new(&BitStream::from_str("abcdefghijklmnopqrstuvwxy"));
        assert_eq!(state.get_lane(0, 0), vec!(0, 1, 1, 0, 0, 0, 0, 1));
        assert_eq!(state.get_lane(4, 4), vec!(0, 1, 1, 1, 1, 0, 0, 1))
    }

    #[test]
    fn test_keccak() {
        let bits = BitStream::from_str("twenty-five-characters ! ");
        let mut k1 = Keccak::new(&bits);
        let mut k2 = Keccak::new(&bits);
        assert_eq!(k1.depth(), 8);
        assert_eq!(k2.depth(), 8);
        k1.keccak(18);
        k2.keccak(18);
        assert_eq!(k1.get_state().as_vec_u64(), k2.get_state().as_vec_u64());
    }

    #[test]
    fn test_theta() {
        let mut k = Keccak::new(&BitStream::from_str(
            "1\0\0\0\01\0\0\0\01\0\0\0\01\0\0\0\01\0\0\0\0",
        ));
        let k1 = Keccak::new(&BitStream::from_val(&[
            0x31, 0x31, 0x00, 0x00, 0x18, 0x31, 0x31, 0x00, 0x00, 0x18, 0x31, 0x31, 0x00, 0x00,
            0x18, 0x31, 0x31, 0x00, 0x00, 0x18, 0x31, 0x31, 0x00, 0x00, 0x18_u8,
        ]));
        k.theta();
        assert_eq!(k.get_state(), k1.get_state());
    }
}
