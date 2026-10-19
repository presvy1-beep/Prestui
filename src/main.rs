use std::io::Write;

use anyhow::Result;
use clap::Parser;

use prestui::{
    app,
    cli::{Cli, Command, RenderMode},
    error::user_error,
    image,
    render::{AnsiRenderer, AsciiRenderer, BlocksRenderer, RenderOptions, Renderer},
    terminal::capabilities,
};

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Render(args)) => render_cli(args),
        None => app::run(cli.image).map_err(|error| anyhow::anyhow!(user_error(None, error))),
    }
}

fn render_cli(args: prestui::cli::RenderArgs) -> Result<()> {
    let image = image::load(&args.image)
        .map_err(|error| anyhow::anyhow!(user_error(Some(&args.image), error)))?;
    let options = RenderOptions {
        width: args.width.unwrap_or(80),
        height: args.height,
        invert: args.invert,
        color: !args.no_color && capabilities::supports_truecolor(),
        charset: args.charset.unwrap_or_default(),
        ..Default::default()
    };
    let rendered = match args.mode {
        RenderMode::Ascii => AsciiRenderer.render(&image, &options),
        RenderMode::Ansi => AnsiRenderer.render(&image, &options),
        RenderMode::Blocks => BlocksRenderer.render(&image, &options),
    };
    std::io::stdout().write_all(rendered.as_bytes())?;
    Ok(())
}
