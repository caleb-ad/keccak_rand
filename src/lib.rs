use std::{
    vec::Vec,
    fmt::{
        Debug,
        Formatter,
        Error,
        Display,
    },
    convert::TryInto,
};

mod bit_stream;
use bit_stream::BitStream;

type Bit = u8;

pub struct Keccak{
    state: Vec<Vec<Vec<Bit>>>,
    w: usize,
    l: u64
}

impl Keccak{

    /// 0<=x<=width, 0<=y<=height, 0<=z<=depth
    pub fn get(& self, x: usize, y: usize, z: usize)->Bit{
        return self.state[x][y][z];
    }

    pub fn get_lane(& self, x: usize, y: usize) -> & Vec<Bit>{
        return &self.state[x][y];
    }

    pub fn depth(& self) -> usize{
        return self.w;
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, val: Bit){
        self.state[x][y][z] = val;
    }

    /// state arrays depth initialized to message length / 25.
    /// If the message is not evenly divisble by 25 then the message size will
    /// be rounded down to the nearest multiple of 25
    pub fn new(message: & BitStream) -> Self{
        let mut temp = Keccak{
            state: Vec::new(),
            w: (message.len() as i64 / 25).try_into().unwrap(),
            l: 0
        };
        temp.l = f64::log2(temp.w as f64) as u64;
        temp.state.resize(5, Vec::new());
        for idx in 0 .. 5 {
            temp.state[idx].resize(5, Vec::new());
            for idx1 in 0 .. 5 {
                temp.state[idx][idx1].resize(temp.w, 0);
            }
        }
        for idx in 0 .. temp.w * 25{
            temp.set((idx / temp.w) % 5, idx / (temp.w * 5), idx % temp.w, message[idx]);
        }
        return temp;
    }

    pub fn get_state(& self) -> BitStream{
        let mut tmp = BitStream::new(self.w * 25);
        for idx in 0..(25 * self.w){
            tmp.set(idx, self.get((idx / self.w) % 5, idx / (self.w * 5), idx % self.w));
        }
        return tmp;
    }

    pub fn empty_state(&self) -> Vec<Vec<Vec<Bit>>>{
        let mut temp = Vec::new();
        temp.resize(5, Vec::new());
        for idx in 0 .. 5 {
            temp[idx].resize(5, Vec::new());
            for idx1 in 0 .. 5 {
                temp[idx][idx1].resize(self.w, 0);
            }
        }
        return temp;
    }

    fn column_parity(&self, x: usize, z: usize) -> Bit{
        return self.state[x][0][z] ^ self.state[x][1][z] ^ self.state[x][2][z]
            ^ self.state[x][3][z] ^ self.state[x][4][z];
    }

    pub fn theta(&mut self){
        for x in 0..(5 as i64){
            for z in 0..(self.w as i64){
                let d = self.column_parity((x-1) as usize %5, z as usize) ^
                    self.column_parity((x+1) as usize %5, (z-1) as usize %(self.w));
                for y in 0..(5 as usize){
                    self.state[x as usize][y][z as usize] ^= d;
                }
            }
        }
    }

    pub fn rho(&mut self){
        let mut new_state = self.empty_state();
        for z in 0..self.w { new_state[0][0][z] = self.state[0][0][z] }
        // could use lookup table to reduce calculations
        let (mut x, mut y) = (1, 0);
        for t in 0..23{
            for z in 0..(self.w as i64){
                new_state[x][y][z as usize] = self.state[x][y][(z - (t+1)*(t+2)/2) as usize % self.w];
            }
            let temp = y;
            y = (2*x + 3*y) % 5;
            x = temp;
        }
        self.state = new_state;
    }

    pub fn pi(&mut self){
        let mut new_state = self.empty_state();
        for x in 0..(5 as usize){
            for y in 0..(5 as usize){
                for z in 0..self.w{
                    new_state[x][y][z] = self.state[(x + 3*y) % 5][x][z];
                }
            }
        }
        self.state = new_state;
    }

    pub fn chi(&mut self){
        let mut new_state = self.empty_state();
        for x in 0..(5 as usize){
            for y in 0..(5 as usize){
                for z in 0..self.w{
                    new_state[x][y][z] = self.state[x][y][z] ^ ((self.state[(x+1)%5][y][z] ^ 1) & self.state[(x+2)%5][y][z]);
                }
            }
        }
        self.state = new_state;
    }

    pub fn rc(t: u64) -> Bit{
        if t % 255 == 0 {return 1;}
        let mut state: u32 = 1;
        for _ in 1 .. t % 255 + 1{
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

    pub fn iota(&mut self, round: u64){
        let mut rc = BitStream::new(self.w);
        for j in 0..self.l{
            rc.set(i64::pow(2, j as u32) as usize - 1, Keccak::rc(j as u64 + 7*round))
        }
        for z in 0..self.w{
            self.state[0][0][z] ^= rc.get(z);
        }
    }

    pub fn keccak(&mut self, num_rounds: i64){
        for r in (12 + 2 * self.l - num_rounds as u64) .. (12 + 2 * self.l){
            self.theta();
            self.rho();
            self.pi();
            self.chi();
            self.iota(r);
        }
    }

}

impl Debug for Keccak{
    fn fmt(& self, f: &mut Formatter<'_>) -> Result<(), Error>{
        for idx in 0..25{
            write!(f, "x:{} y:{}, {:?}\n", idx % 5, idx / 5, self.get_lane(idx % 5, idx / 5));
        }
        return Ok(());
    }
}

/// allows use of ToString
impl Display for Keccak{
    fn fmt(& self, f: &mut Formatter<'_>) -> Result<(), Error>{
        return Debug::fmt(self, f);
    }
}

#[cfg(test)]
mod tests {

    use crate::Keccak;
    use crate::bit_stream::BitStream;
    use std::mem::size_of;

    #[test]
    fn test_bitstream_get() {
        let bits = BitStream::from_i64(0x5d);
        assert_eq!(bits.len(), 64);
        assert_eq!(bits.get(0), 0);
        assert_eq!(bits.get(1), 0);
        assert_eq!(bits.get(59), 1);
        assert_eq!(bits.get(58), 0);
        assert_eq!(bits.get(63), 1);
        assert_eq!(bits.get(62), 0);
    }

    #[test]
    fn test_bitstream_from_i64() {
        let bits = BitStream::from_i64(0x1234cdef);
        let mut result: i64 = 0;
        assert_eq!(bits.len(), 64);
        for idx in 0..bits.len(){
            result |= (bits.get(idx) as i64) << (63 - idx);
        }
        assert_eq!(result, 0x1234cdef);
    }

    #[test]
    fn test_bitstream_from_usize() {
        let bits = BitStream::from_usize(0x6789abcd);
        let mut result: usize = 0;
        assert_eq!(bits.len(), size_of::<usize>() * 8);
        for idx in 0..size_of::<usize>()*8{
            result |= (bits.get(idx) as usize) << (size_of::<usize>()*8 - 1 - idx);
        }
        assert_eq!(result, 0x6789abcd);
    }

    #[test]
    fn test_bitstream_from_str() {
        let bits = BitStream::from_str("abcdefgh");
        assert_eq!(bits.len(), 64);
        let mut result: i64 = 0;
        for idx in 0..bits.len(){
            result |= (bits.get(idx) as i64) << 63 - idx;
        }
        assert_eq!(result, 0x6162636465666768);
    }

    #[test]
    fn test_keccak_new() {
        // create 200 bit, 5*5*8 state array
        let state = Keccak::new(& BitStream::from_str("abcdefghijklmnopqrstuvwxy"));
        assert_eq!(*state.get_lane(0, 0), vec!(0,1,1,0,0,0,0,1));
        assert_eq!(*state.get_lane(4, 4), vec!(0,1,1,1,1,0,0,1))
    }

    #[test]
    fn test_keccak() {
        let bits = BitStream::from_str("twenty-five-characters ! ");
        let mut k1 = Keccak::new(& bits);
        let mut k2 = Keccak::new(& bits);
        assert_eq!(k1.depth(), 8);
        assert_eq!(k2.depth(), 8);
        k1.keccak(18);
        k2.keccak(18);
        assert_eq!(k1.get_state().as_vec_u8(), k2.get_state().as_vec_u8());
    }
}
