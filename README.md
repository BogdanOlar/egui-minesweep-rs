
# minesweep-rs

A rust implementation of the popular game, using the [egui](https://github.com/emilk/egui) library.

![screenshot](.github/Screenshot.png)

## Build & run

### Desktop

Prerequisites: `cargo` and `rustc` (see [installation instructions](https://www.rust-lang.org/tools/install))

```bash
git clone https://github.com/BogdanOlar/minesweep-rs.git
cd minesweep-rs/
cargo run --release
```

### Wasm

Prerequisites: execute the setup script before the first build

```bash
./setup_web.sh
```

Build and run:

```bash
./start_web.sh
```

The output should look something like this:

```bash
   Compiling minesweep-rs v0.1.0 (/home/bogdan/Workspace/Projects/minesweep-rs)
    Finished release [optimized] target(s) in 1.98s
[INFO ] basic-http-server 0.8.1
[INFO ] addr: http://127.0.0.1:3000
[INFO ] root dir: .
[INFO ] extensions: false
```

The app should be available on [localhost port 3000](http://127.0.0.1:3000)

## TODO

- [X] Linux
- [X] WASM

- [ ] `egui` layout
- [X] Config

## License

[MIT](./LICENSE)
