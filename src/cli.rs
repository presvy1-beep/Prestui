use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "prestui", version, about = "Render images in the terminal", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(help = "Image to open in the interactive viewer")]
    pub image: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Render an image directly to stdout")]
    Render(RenderArgs),
}

#[derive(clap::Args, Debug)]
pub struct RenderArgs {
    #[arg(help = "Image path")]
    pub image: PathBuf,

    #[arg(short, long, value_enum, default_value_t = RenderMode::Ascii, help = "Renderer to use")]
    pub mode: RenderMode,

    #[arg(
        short,
        long,
        value_parser = clap::value_parser!(u32).range(1..=1000),
        help = "Output width in terminal cells (1-1000)"
    )]
    pub width: Option<u32>,

    #[arg(
        short = 'H',
        long,
        help = "Output height in terminal cells (for blocks, this is cell rows)"
    )]
    pub height: Option<u32>,

    #[arg(short, long, help = "Invert luminance for ASCII rendering")]
    pub invert: bool,

    #[arg(long, help = "Disable ANSI color escape sequences")]
    pub no_color: bool,

    #[arg(long, help = "Character ramp, ordered from dark to light")]
    pub charset: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum RenderMode {
    Ascii,
    Ansi,
    Blocks,
}
