# PresTUI

![PresTUI demo](https://raw.githubusercontent.com/presvy1-beep/Prestui/v0.1.0/prestui.gif)

## Install (Linux x86-64)

Download and install the [v0.1.0 release](https://github.com/presvy1-beep/Prestui/releases/tag/v0.1.0):

```sh
curl -fLO https://github.com/presvy1-beep/Prestui/releases/download/v0.1.0/prestui-v0.1.0-linux-x86_64.tar.gz
curl -fLO https://github.com/presvy1-beep/Prestui/releases/download/v0.1.0/prestui-v0.1.0-linux-x86_64.tar.gz.sha256
sha256sum -c prestui-v0.1.0-linux-x86_64.tar.gz.sha256
tar -xzf prestui-v0.1.0-linux-x86_64.tar.gz
sudo install -Dm755 prestui-v0.1.0-linux-x86_64/prestui /usr/local/bin/prestui
```

Requires a compatible glibc-based x86-64 Linux system.

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
