use std::io;
use std::fs::File;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rhtail",
            author = "RonHass",
            about = "output the last part of files")]
struct CLI {
    #[structopt(short = "-n", long, default_value = "10")]
    lines: u64,

    file_path: String,

    #[structopt(short, long)]
    follow: bool,
}

fn main() -> Result<(), io::Error>{
    let args = CLI::from_args();
    let mut file = File::open(args.file_path)?;
    rhtail::tail(&mut file, &mut io::stdout(), args.lines)?;
    if args.follow {
        rhtail::follow_file(&mut file, &mut io::stdout())?;
    }
    Ok(())
}
