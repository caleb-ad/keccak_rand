use rand_keccak::Keccak;
use rand_keccak::BitStream;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;
use std::time::Duration;

fn gen_sample(size: u64, range_max: u64, seed: u64) -> Vec<u64>{
   let mut temp = Vec::new();
   let mut generator = Keccak::new_sized(& BitStream::from_u64(&[seed]), 8);
   for _ in 0..size{
      temp.push(generator.copy_to_u64() % range_max);
      generator.keccak(18);
   }
   return temp;
}

fn gen_sample_to_file(size: u32, range_max: u64, seed: u64){
   let mut temp = File::create("test_result.csv").unwrap();
   let mut generator = Keccak::new_sized(& BitStream::from_u64(&[seed]), 8);
   for _ in 0..size{
      temp.write(format!("{},\n ", generator.copy_to_u64() % range_max).as_bytes()).unwrap();
      generator.keccak(18);
   }
}

fn get_mean(sample: & Vec<u64>) -> f64{
   let mut sum = 0;
   for idx in 0..sample.len(){
      sum += sample[idx];
   }
   return sum as f64 / sample.len() as f64;
}

fn get_std_deviation(sample:& Vec<u64>, mean: f64) -> f64{
   let mut sum = 0.0;
   for idx in 0..sample.len(){
      sum += (sample[idx] as f64 - mean) * (sample[idx] as f64 - mean);
   }
   return f64::sqrt(sum / sample.len() as f64);
}

/// in expected distribution each value in [0, range_max) occurrs an equal
/// number of times
fn std_dev_exp(range_max: usize) -> f64{
   let mean = (range_max - 1) as f64 / 2.0;
   let mut sum = 0.0;
   for val in 0..range_max{
      sum += (mean - val as f64) * (mean - val as f64);
   }
   return f64::sqrt(sum / range_max as f64);
}

fn get_maxmin(sample:& Vec<u64>) -> (u64, u64){
   // need non-zero sample size
   let mut min = sample[0];
   let mut max = sample[0];
   for idx in 1..sample.len(){
      match sample[idx]{
         val if val < min => min = val,
         val if val > max => max = val,
         _ => continue,
      }
   }
   return (min, max);
}

#[allow(unused_macros)]
macro_rules! print_desc_stats {
   ($mean:expr, $sd:expr, $max:expr, $min:expr) => {
      println!("Distribution Description:
      Mean: {}
      Std Dev: {}
      Max: {}
      Min: {}", $mean, $std_dev, $max, $min);
   };
}

fn test_sample(range_max: u64, sample_size: u64){
   // below error bounds for mean and standard deviation were chosen so the test can scale
   // to dfferent range and sample sizes, but without any theoretical backing
   let sample = gen_sample(sample_size, range_max, SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
   // expected mean = (range_max - 1) / 2
   let mean = get_mean(& sample);
   assert_eq!(mean - ((range_max as f64 - 1.0) / 2.0) < (range_max as f64 / sample_size as f64), true);
   let std_dev = get_std_deviation(& sample, mean);
   assert_eq!(std_dev - std_dev_exp(range_max as usize) < (range_max as f64 / sample_size as f64), true);
}

#[test]
fn test_randomness() {
    for range in [2, 5, 100, 1000, 1000000]{
       test_sample(range, 1000);
    }
}

/*
#[test]
fn generate_test_file() {
    gen_sample_to_file(1000, 100, SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
}
*/
