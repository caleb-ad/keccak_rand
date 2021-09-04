/// bits should only be 1 or 0
type Bit = u8;

use std::{
   mem::size_of,
   fmt::{
      Debug,
      Display,
      Formatter,
      Error,
   },
   ops::{
      Index,
   },
   convert::Into,
   convert::TryInto,
};

#[derive(Debug, PartialEq)]
pub struct BitStream{
   bits: Vec<u64>,
   length: usize
}

impl BitStream{
   const ONE: u8 = 1;
   const ZERO: u8 = 0;

   /// for i64 and usize, bits are in "reverse" order, ie. get(0) returns MSB
   pub fn from_u64(src:& [u64]) -> Self{
      let mut temp = BitStream{
         bits: Vec::new(),
         length: 64 * src.len(),
      };
      temp.bits = src.to_vec();
      return temp;
   }

   pub fn from_val<T>(src:& [T]) -> Self
   where T: Into<u64> + Copy
   {
      let mut temp = BitStream{
         bits: Vec::<u64>::new(),
         length: size_of::<T>() * 8 * src.len(),
      };
      temp.bits.resize(f64::ceil((size_of::<T>() * src.len()) as f64 / 8.0) as usize, 0);
      let mut idx = 0;
      let mut b_idx: i32 = 56;
      for src_idx in 0..src.len(){
         for src_b_idx in (0..size_of::<T>()).rev(){
            temp.bits[idx] |= ((0xFF << src_b_idx * 8) & src[src_idx].into()) << (b_idx as usize - src_b_idx * 8);
            b_idx -= 8;
            if b_idx < 0{
               b_idx = 56;
               idx += 1;
            }
         }
      }
      return temp;
   }

   pub fn try_from_val<T>(src:& [T]) -> Self
   where T: TryInto<u64> + Copy,
         <T as TryInto<u64>>::Error: Debug
   {
      let mut temp = BitStream{
         bits: Vec::<u64>::new(),
         length: size_of::<T>() * 8 * src.len(),
      };
      temp.bits.resize(f64::ceil((size_of::<T>() * src.len()) as f64 / 8.0) as usize, 0);
      let mut idx = 0;
      let mut b_idx: i32 = 56;
      for src_idx in 0..src.len(){
         for src_b_idx in (0..size_of::<T>()).rev(){
            temp.bits[idx] |= ((0xFFu64 << src_b_idx * 8) & src[src_idx].try_into().unwrap() ) << (b_idx as usize - src_b_idx * 8);
            b_idx -= 8;
            if b_idx < 0{
               b_idx = 56;
               idx += 1;
            }
         }
      }
      return temp;
   }

   pub fn add_val<T>(&mut self, src:& [T]) -> ()
   where T: Into<u64> + Copy
   {
      let mut idx = self.bits.len() - 1;
      let mut b_idx: i32 = (self.len() as u64 % 64) as i32;
      assert_eq!(b_idx % 8, 0);
      for src_idx in 0..src.len(){
         for src_b_idx in (0..size_of::<T>()).rev(){
            self.bits[idx] |= ((0xFFu64 << src_b_idx * 8) & src[src_idx].into()) << (b_idx as usize - src_b_idx * 8);
            b_idx -= 8;
            if b_idx < 0{
               b_idx = 56;
               idx += 1;
               self.bits.push(0);
            }
         }
      }
   }

   pub fn try_add_val<T>(&mut self, src:& [T]) -> ()
   where T: TryInto<u64> + Copy,
         <T as TryInto<u64>>::Error: Debug
   {
      let mut idx = self.bits.len() - 1;
      let mut b_idx: i32 = (self.len() as u64 % 64) as i32;
      assert_eq!(b_idx % 8, 0);
      for src_idx in 0..src.len(){
         for src_b_idx in (0..size_of::<T>()).rev(){
            self.bits[idx] |= ((0xFFu64 << src_b_idx * 8) & src[src_idx].try_into().unwrap()) << (b_idx as usize - src_b_idx * 8);
            b_idx -= 8;
            if b_idx < 0{
               b_idx = 56;
               idx += 1;
               self.bits.push(0);
            }
         }
      }
   }

   pub fn from_str<'a>(src: &'a str) -> Self{
      let mut temp = BitStream{
         bits: Vec::new(),
         length: src.len() * 8,
      };
      temp.bits.resize(f64::ceil(src.len() as f64 / 8.0) as usize, 0);
      let mut idx = 0;
      let mut b_idx: i32 = 56;
      for byte in src.as_bytes(){
         temp.bits[idx] |= (*byte as u64) << b_idx;
         b_idx -= 8;
         if b_idx < 0 {
            b_idx = 56;
            idx += 1;
         }
      }
      return temp;
   }

   /// new bitstream of length zero-initialized bits
   pub fn new(length: usize) -> Self{
      let mut temp = BitStream{
         bits: Vec::new(),
         length: length,
      };
      temp.bits.resize(f64::ceil(length as f64 / 8.0) as usize, 0);
      return temp;
   }

   pub fn get(& self, idx: usize) -> Bit{
      if idx >= self.length {panic!("BitStream index out of bounds")};
      return (self.bits[idx / 64] >> (63 - idx % 64)) as u8 & 1;
   }

   /// only the least significant bit of val has any effect, so function is
   /// safe to use even if val isn't a true bit.
   pub fn set(&mut self, idx: usize, val: Bit){
      let setter: u64 = 0x7FFFFFFFFFFFFFFF;
      self.bits[idx / 64] = (self.bits[idx / 64] & (setter >> (idx % 64))) | ((val as u64 & 1) << 63 - idx % 64);
   }

   pub fn len(& self) -> usize{
      return self.length;
   }

   pub fn as_vec_u64(& self) -> Vec<u64>{
      return self.bits.clone();
   }

}

impl Display for BitStream{
   fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>{
      for idx in 0 .. self.len(){
         match write!(f, "{:b}", self.bits[idx]){
            Err(err) => return Err(err),
            _ => continue,
         }
      }
      match write!(f, "\n"){
         Err(err) => Err(err),
         _ => Ok(()),
      }
   }
}

impl Index<usize> for BitStream{
   type Output = Bit;
   fn index(&self, idx: usize) -> & Self::Output{
      if idx >= self.length {panic!("BitStream index out of bounds")};
      match (self.bits[idx / 64] >> (63 - idx % 64)) & 1{
         0 => & BitStream::ZERO,
         _ => & BitStream::ONE,
      }
   }
}
