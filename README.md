# PresTUI

![PresTUI demo](./prestui.gif)

PresTUI renders images in the terminal as ASCII, ANSI TrueColor, or Unicode
half blocks. It also includes an interactive Ratatui viewer.

## Install

Requires Rust and Cargo. Install from source:

```sh
cargo install --git https://github.com/presvy1-beep/PresTUI
```

Or clone and build locally:

```sh
git clone https://github.com/presvy1-beep/PresTUI
cd PresTUI
cargo build --release
```

## Use

Render an image to stdout:

```sh
prestui render image.png --mode ascii --width 80
prestui render image.png --mode ansi --width 100
prestui render image.png --mode blocks --width 100
prestui render image.png --mode ascii > image.txt
```

Run `prestui` to open the image picker, or pass an image to open it directly:

```sh
prestui
prestui image.png
```

In the viewer:

```text
1 / 2 / 3   ASCII / ANSI / Blocks
+ / -       Change preview width
i           Invert ASCII luminance
a           Toggle aspect correction
o           Open an image
s           Copy rendered output
?           Toggle help
q / Esc     Quit
```

The viewer responds to terminal resizing. CLI output goes to stdout and errors
go to stderr. Run `prestui render --help` for all options.

PNG, JPEG, WebP, BMP, and GIF images are supported. See [LICENSE](LICENSE) for
the MIT license.

For development, run `cargo fmt --all -- --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --locked`.
