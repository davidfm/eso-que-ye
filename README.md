# Eso qué ye?

A web app for recognising and transcribing traditional Asturian instrumental folk music. It listens to a melody, transcribes it to sheet music, and searches the [AsturTrad](https://asturtrad.eu) tune library for matches. Runs entirely in the browser with no internet connection required.

> This project is a fork of [FolkFriend](https://github.com/TomWyllie/folkfriend)
> by [Tom Wyllie](https://github.com/TomWyllie), adapted to use the AsturTrad library.
> All credit for the recognition engine, architecture, and original app goes to Tom.
> The original app lives at [folkfriend.app](https://folkfriend.app).

## License

This project is licensed under [GPL-3.0](LICENSE). As required by the license, the source code is open, all original copyright notices are preserved, and modifications are noted.

---

## Getting started

### Prerequisites

Install [mise](https://mise.jdx.dev) to manage Rust and Node:

```bash
curl https://mise.run | sh
# Follow the prompt to activate mise in your shell, then:
mise install
```

### Setup

```bash
mise run setup      # install wasm-pack and npm dependencies (once)
mise run wasm       # compile Rust → WebAssembly (re-run when Rust code changes)
```

Place your tune index files in `app/public/`:

```bash
cp /path/to/folkfriend-non-user-data.json app/public/
cp /path/to/nud-meta.json app/public/
```

### Development server

```bash
mise run dev        # starts at http://localhost:8080
```

### Production build

```bash
mise run build      # output in app/dist/
```

---

## Repository structure

| Directory | Description |
|---|---|
| `app/` | Vue.js PWA source code |
| `rust/` | Recognition engine source code (Rust, compiled to WASM) |
| `rust/wavs/` | Sample audio files for testing |

