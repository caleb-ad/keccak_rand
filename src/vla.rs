#![allow(dead_code)]

use std::{
   alloc::{
       alloc_zeroed,
       dealloc,
       Layout,
   },
};

/// look into using array::from_raw instead of raw pointer
pub struct VLA<T>{
   len: usize,
   data: *mut T,
}

impl<T: Copy> VLA<T>{
   pub fn new(length: usize) -> Self{
   unsafe{
       let temp = VLA{
           len: length,
           data: alloc_zeroed(Layout::array::<T>(length).unwrap()) as *mut T,
       };
       if temp.data.is_null(){
           panic!("allocation failed");
       }
       return temp;
   }
   }

   pub fn get(& self, idx: usize) -> T{
       //slowest part of function, but could have undefined dereference withou it
       assert_eq!(idx < self.len, true);
       return unsafe{ *self.data.add(idx) };
   }


   pub fn set(&mut self, idx: usize, val: T){
      unsafe{
          assert_eq!(idx < self.len, true);
          *self.data.add(idx) = val;
      }
   }

   pub fn len(& self) -> usize{
       return self.len;
   }
}

impl<T> Drop for VLA<T>{
   fn drop(&mut self){
       unsafe{
           dealloc(self.data as *mut u8, Layout::array::<T>(self.len).unwrap());
       }
   }
}

impl<T: PartialEq> PartialEq for VLA<T>{
   fn eq(& self, other: &Self) -> bool{
      if self.len != other.len {return false};

      for idx in 0..self.len{ unsafe{
         if *(self.data.add(idx)) != *(other.data.add(idx)) {return false};
      }}
      return true;
   }
}
