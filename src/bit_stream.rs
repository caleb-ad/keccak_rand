/// bits should only be 1 or 0
type Bit = u8;

use std::{
   mem::size_of,
   fmt::{
      Debug,
      Formatter,
      Error
   },
   ops::{
      Index,
      BitAnd,
      BitOr,
      BitAndAssign,
      BitOrAssign,
      Shl,
      ShlAssign,
      Shr,
      ShrAssign,
   },
   convert::Into,
};

pub struct BitStream<I>
//where I: BitAnd + BitOr + BitAndAssign + BitOrAssign +
//         Shl + Shr + ShlAssign + ShrAssign + Debug
{
   bits: Vec<I>,
   unit_size: usize, // in bytes
   length: usize // in bits
}

impl<I> BitStream<I>
where I: BitAnd + BitOr + BitAndAssign + BitOrAssign +
         Shl + Shr + ShlAssign + ShrAssign + Debug + Clone + From<i64>
{
   const ONE: u8 = 1;
   const ZERO: u8 = 0;

   /// integer types are put in the bit stream from MSB -> LSB
   pub fn from_i64(src: i64) -> Self{
      let mut temp = BitStream{
         bits: Vec::new(),
         length: size_of::<i64>() * 8,
      };
      temp.bits.resize(size_of::<i64>(), 0);
      let mask: i64 = 0xFF;
      for idx in 0..8{
         temp.bits[idx] = ((src >> (7 - idx)*8) & mask) as u8;
      }
      return temp;
   }

   pub fn from_usize(src: usize) -> Self{
      let mut temp = BitStream{
         bits: Vec::<I>::new(),
         unit_size: size_of::<I>(),
         length: size_of::<usize>() * 8,
      };
      temp.bits.resize(temp.unit_size / size_of::<usize>(), I::from(0));
      let mut unit_idx = 0;
      let mut idx = 0;
      for b_idx in (0..size_of::<usize>()).rev(){
         temp.bits[idx] &= I::from((src & (0xFF_usize << b_idx)) as i64 >> unit_idx);
         unit_idx -= 1;
         if unit_idx == 0 {
            unit_idx = temp.unit_size;
            idx += 1;
         }
      }
      return temp;
   }

   pub fn from_val(src:& [I]) -> Self{
      let mut temp = BitStream{
         bits: Vec::<I>::new(),
         unit_size: size_of::<I>(),
         length: size_of::<usize>() * 8,
      };
   }

   pub fn from_str<'a>(src: &'a str) -> Self{
      let mut temp = BitStream{
         bits: Vec::<I>::new(),
         unit_size: size_of::<I>(),
         length: src.len() * 8,
      };
      temp.bits.resize(temp.unit_size / src.len(), I::from(0));
      let mut idx = 0;
      let mut unit_idx = temp.unit_size - 1;
      for byte in src.as_bytes(){
         temp.bits[idx] &= I::from((*byte as i64) << unit_idx * 8);
         unit_idx -= 1;
         if unit_idx == 0 {
            unit_idx = temp.unit_size;
            idx += 1;
         }
      }
      return temp;
   }

   /// new bitstream of length zero-initialized bits
   pub fn new(length: usize) -> Self{
      let mut temp = BitStream{
         bits: Vec::<I>::new(),
         unit_size: size_of::<I>(),
         length: length,
      };
      temp.bits.resize(f64::ceil(length as f64 / 8.0) as usize, I::from(0));
      return temp;
   }

   pub fn get(& self, idx: usize) -> Bit{
      if idx >= self.length {panic!("BitStream index out of bounds")};
      return (self.bits[idx / 8] >> (7 - idx % 8)) & 1;
   }

   /// only the least significant bit of val has any effect, so function is
   /// safe to use even if val isn't a true bit.
   pub fn set(&mut self, idx: usize, val: Bit){
      self.bits[idx / 8] = (self.bits[idx / 8] & (0b1111111101111111u16 >> (idx % 8)) as u8) | ((val & 1) << 7 - idx % 8);
   }

   pub fn len(& self) -> usize{
      return self.length;
   }

   pub fn as_vec_u8(& self) -> Vec<u8>{
      return self.bits.clone();
   }

}

impl Debug for BitStream{
   fn fmt(&self, _f: &mut Formatter<'_>) -> Result<(), Error>{
      for idx in 0 .. self.len(){
         print!("{}", self.get(idx) as u8);
      }
      println!("");
      return Ok(());
   }
}

impl Index<usize> for BitStream{
   type Output = Bit;
   fn index(&self, idx: usize) -> & Self::Output{
      if idx >= self.length {panic!("BitStream index out of bounds")};
      match (self.bits[idx / 8] >> (7 - idx % 8)) & 1{
         0 => & BitStream::ZERO,
         _ => & BitStream::ONE,
      }
   }
}

/* impl PartialEq for BitStream{
   fn eq(& self, other: & Self) -> bool{
      return self.bits == other.bits;
   }
} */
