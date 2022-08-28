use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rhtail",
            author = "RonHass",
            about = "output the last part of files")]
struct CLI {
    #[structopt(short = "-n", long, default_value = "10")]
    lines: u64,

    file_path: String,
}

fn main() {
    let args = CLI::from_args();
    if let Err(e) = rhtail::tail(args.file_path, args.lines) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
