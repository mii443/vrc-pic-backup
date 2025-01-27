use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub src: String,
    pub dst: String,
    #[clap(short, long, default_value = "80")]
    pub quality: f32,
    #[clap(short, long)]
    pub threads: Option<usize>,
}
