use std::vec;

fn modify_vec(start: usize, vector: &mut vec::Vec<u64>){
   for x in 0..vector.len(){
      vector[x] = (x + start) as u64;
   }
}

fn modify_arr(start: usize, array: &mut [u64]){
   for x in 0..array.len(){
      array[x] = (x + start) as u64;
   }
}

fn main(){
   let mut my_array = [0_u64; 10000];
   let mut my_vec: vec::Vec<u64> = vec::Vec::new();
   my_vec.resize(10000, 0);
   for x in 0..10000{
      // according to Intel VTune, the CPU spends twice as much time in modify_vec
      // then it does in modify_arr
      modify_arr(x, &mut my_array);
      modify_vec(x, &mut my_vec);
   }
}