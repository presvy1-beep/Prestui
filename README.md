# PresTUI

![PresTUI demo](https://raw.githubusercontent.com/presvy1-beep/Prestui/v0.1.0/prestui.gif)

## Download and Install

Requires Rust and Cargo.

```sh
cargo install prestui
```

## Usage

```sh
prestui
prestui image.png
```

Render an image directly in the terminal:

```sh
prestui render image.png --mode ascii --width 80
prestui render image.png --mode ansi --width 100
prestui render image.png --mode blocks --width 100
```
