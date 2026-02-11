use crate::player::{Player, PlayerTrait};
use crate::strategies::Strategy;

pub struct Part1 {}

// Terrible strategy: ask if the number is min, otherwise return max.
impl Strategy for Part1 {
    fn guess_the_number(player: &mut Player, min: u32, max: u32) -> u32 {
        let mut min_guess = min;
        loop{
            if min_guess < max{
                if player.ask_if_equal(min_guess){
                return min_guess;
            } else{
                min_guess+=1;
                if min_guess >= max{
                    println!("Out of range");
                    return min;
                }
            }
            }
        }
    }
}
