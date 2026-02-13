use crate::player::{Player, PlayerTrait};
use crate::strategies::Strategy;

pub struct Part2 {}

// Terrible strategy: ask if the number is min, otherwise return max.
impl Strategy for Part2 {
    fn guess_the_number(player: &mut Player, min: u32, max: u32) -> u32 {
        let mut low = min;
        let mut high = max;
        loop{
            let mid = low + (high - low) / 2;
            match player.ask_to_compare(mid) {
                0 => return mid,
                1 => {
                    low = mid + 1;
                }
                _ => {
                    if mid == 0 {
                        return 0;
                    }
                    high = mid - 1;
                }
            }

            if low > high {
                println!("Out of range");
                return low;
            }
        }
    }
}