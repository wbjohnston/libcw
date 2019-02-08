use failure::{Error, Fail};
use itertools::Either;
use rand::Rng;
use redcode::{Address, Instruction};
use simulation::{Mars, Pid};
use std::collections::HashMap;

pub type Pin = usize;

#[derive(Debug, Clone, Copy, Fail)]
pub enum GameError {
  #[fail(display = "conflicting pin: {}", pin)]
  PinConflict { pin: Pin },
}

/// Corewars game runtime that wraps a `Mars` to provide additional information
/// about the game
#[derive(Debug, Clone, Default)]
pub struct Game {
  // associate player pins with
  pin_to_pid: HashMap<Pin, Pid>,
  mars: Mars,
}

impl Game {
  /// Add a player to the game with a pin
  pub fn add_player_with_pin(
    &mut self,
    program: &[Instruction],
    address: Address,
    pin: Pin,
  ) -> Result<Pin, GameError> {
    // if there is a pin conflict
    if self.pin_to_pid.get(&pin).is_some() {
      Err(GameError::PinConflict { pin })
    } else {
      let pid = self.mars.load_program(program, address);
      self.pin_to_pid.insert(pin, pid);
      Ok(pin)
    }
  }

  /// Add a player to the game
  pub fn add_player(
    &mut self,
    program: &[Instruction],
    address: Address,
  ) -> Result<Pin, GameError> {
    let pin = self.gen_next_pin();
    self.add_player_with_pin(program, address, pin)
  }

  /// Add a player to the game with a pin, loaded at a random location
  pub fn add_player_with_pin_rand<R>(
    &mut self,
    program: &[Instruction],
    pin: Pin,
    rng: &mut R,
  ) -> Result<Pin, GameError>
  where
    R: Rng,
  {
    // TODO: add "minimum margin" between programs
    let load_addr = rng.gen();
    self.add_player_with_pin(program, load_addr, pin)
  }

  /// Add a player to the game, loaded at a random location
  pub fn add_player_rand<R>(
    &mut self,
    program: &[Instruction],
    rng: &mut R,
  ) -> Result<Pin, GameError>
  where
    R: Rng,
  {
    // TODO: add "minimum margin" between programs
    let load_addr = rng.gen();
    self.add_player(program, load_addr)
  }

  /// Step the game forward one turn and return `Some(pin)` if the player with
  /// the `pin` as a pin was eliminated. Otherwise `None`
  pub fn step(&mut self) -> Option<Pin> {
    self.mars.step().and_then(|ref pid| {
      // NOTE: is this unwrap ok? I feel like it is
      let pin = self
        .pin_to_pid
        .get(&pid)
        .expect("Somehow executed with killed with process loaded without a pin");
      Some(*pin)
    })
  }

  /// Return pins associated with their owned process id
  pub fn pins_with_pids(&self) -> impl Iterator<Item = (&Pin, &Pid)> {
    self.pin_to_pid.iter()
  }

  pub fn winner(&self) -> Option<Pin> {
    if self.mars.process_count() == 1 {
      self.mars.pid()
    } else {
      None
    }
  }

  pub fn mars(&self) -> &Mars {
    &self.mars
  }

  /// Return the next available pin
  fn gen_next_pin(&mut self) -> Pin {
    let mut pin = 0;

    loop {
      if !self.pin_to_pid.contains_key(&pin) {
        break pin;
      }

      pin += 1;
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use redcode::{AddressingMode::*, OpCode::*, OpMode::*, *};

  #[test]
  fn single_process_is_winner() {
    let program = &[Instruction::new(Mov, I, Direct, 0, Direct, 1)];
    let mut game = Game::default();
    let pid = game.add_player(program, 0).unwrap();
    assert_eq!(pid, game.winner().unwrap());

    // verify that there is no winner when another player is added
    game.add_player(program, 0).expect("should not conflict");
    assert_eq!(None, game.winner());
  }
}
