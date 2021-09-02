use bit_stream::BitStream;
use keccak_rand::Keccak;

fn gen_sample(size: u32, range_max: u64) -> Vec<u64>{
   let temp = Vec::new();
   let generator = Keccak::new_sized(BitStream::from_u64(0xdeadbeef), 8);
   for _ in 0..size{
      temp.push(generator.copy_to_u64());
      generator.keccak();
   }
   return temp;
}