use clap::Parser;
use tcproxy_cli::{ClientArgs, App, Type};
use tcproxy_core::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ClientArgs::parse();
    match args.get_type() {
        Type::Listen(args) => {
            App::new(args.clone())
                .start()
                .await?;
        },
        Type::Config => {
            println!("received config command");
        }
    }

    Ok(())
}
