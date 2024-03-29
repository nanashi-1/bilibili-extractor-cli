# bilibili-extractor-cli

*Do not use this project anymore! Use [bilibili-extractor](https://github.com/nanashi-1/bilibili-extractor) instead.*

This is a CLI tool for the library [bilibili-extractor](https://github.com/nanashi-1/bilibili-extractor).

## Installation

### Build

    cargo install --git https://github.com/nanashi-1/bilibili-extractor-cli

## Usage

1. Locate the Bilibili download directory. (location: "`Internal Storage/Android/data/com.bstar.intl/download`")

![Bilibili Folder](assets/bilibili-folder.png)

2. Copy the Bilibili download directory to a safe place

![Safe Place](assets/paste-download.png)

3. Launch terminal in that directory

![Open Terminal](assets/open-in-console.png)

4. Run this command

![Run Command](assets/run-command.png)

> Note: `-d` here forces the program to move the output instead of copying it resulting in faster packaging. You can also specify `-H` or `--hard-sub` to merge subtitle into the video as a hard sub.

## Lisence

This project is licensed under the MIT License.
