use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct ClientArgs {
  #[clap(short, long, value_parser, default_value = "8080")]
  port: u16,

  #[clap(short, long, value_parser, default_value = "0.0.0.0")]
  ip: String,

  #[clap(long, value_parser, default_value = "false")]
  debug: bool
}

impl ClientArgs {
  pub fn is_debug(&self) -> bool {
    self.debug
  }
}