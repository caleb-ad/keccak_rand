use rand_keccak::Keccak;
use rand_keccak::BitStream;

fn gen_sample(size: u32, range_max: u64) -> Vec<u64>{
   let mut temp = Vec::new();
   let mut generator = Keccak::new_sized(& BitStream::from_u64(&[0xdeadbeef]), 8);
   for _ in 0..size{
      temp.push(generator.copy_to_u64() % range_max);
      generator.keccak(18);
   }
   return temp;
}

fn main(){
   //5.652s
   gen_sample(1000, 100);
}
