use std::io::Read;
use tun_playground::argument_parser;
#[tokio::main]
async fn main() {
    let args = argument_parser::parse_args();
    args.exec().await;
}
