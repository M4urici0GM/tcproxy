use std::sync::Arc;

use async_trait::async_trait;
use tcproxy_core::{Command, Result};


struct GetPortCommand;

#[async_trait]
impl Command for GetPortCommand {
  type Output = Result<u16>;

  fn handle(&mut self) -> Self::Output {
    todo!() 
  }
}