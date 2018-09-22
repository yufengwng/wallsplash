### wallsplash

a rust app for spicing up your desktop by rotating the wallpaper using images
in a local directory and images downloaded from unsplash

[![Build Status](
https://travis-ci.org/yufengwng/wallsplash.svg?branch=master)](
https://travis-ci.org/yufengwng/wallsplash)

\# setup

1. sign-up on [unsplash], create a new app, and copy the access key
2. install `feh` using your package manager

[unsplash]: https://unsplash.com/developers

\# install

1. `git clone <repo>`
2. `cd <repo>`
3. `cargo build --release`
4. `cp ./target/release/wallsplash /some/where/in/your/path`

\# run

1. grab the config file: `cp ./conf/example.toml ~/.config/wallsplash/config.toml`
2. edit config file, paste in unsplash access key
3. run it in the background: `/path/to/wallsplash >/dev/null 2>&1 &!`

