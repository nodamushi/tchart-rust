# tchart-rust

English | [日本語](README.ja.md)

A Rust reimplementation of the [original tchart](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/indexframe.html).
Converts TCML (Timing Chart Markup Language) text into SVG timing charts.

No external runtime such as Node.js. Runs on Linux and Windows.

## Samples

![sample output](docs/images/sample.png)
![arrows](docs/images/arrow.png)
![clock markers](docs/images/clock_marks.png)

Source TCML: [`docs/images/sample.tc`](docs/images/sample.tc) / [`docs/images/arrow.tc`](docs/images/arrow.tc) / [`docs/images/clock_marks.tc`](docs/images/clock_marks.tc)

## Quick start

### CLI

```bash
cargo build --release -p tchart-cli
target/release/tchart svg chart.tc
target/release/tchart png chart.tc
```

See [`docs/spec/cli.md`](docs/spec/cli.md) for details.

### Web editor

Hosted on GitHub Pages — no install required:

- Editor: <https://nodamushi.github.io/tchart-rust/>
- Help (English): <https://nodamushi.github.io/tchart-rust/help/tcml-format.en.html>
- Help (Japanese): <https://nodamushi.github.io/tchart-rust/help/tcml-format.html>

The released editor is also a single self-contained HTML file (`tchart-editor/dist/index.html`). Drop it on your desktop and double-click — no Node, Rust, or network access required at runtime; CSS, JS, the wasm module, and the help content are all inlined.

To build it from source:

```bash
wasm-pack build tchart-web --target web
cd tchart-editor
pnpm install
pnpm build           # produces dist/index.html (single file)
pnpm dev             # local dev server
```

### WASM library

```typescript
import init, { render_tcml } from './tchart-web/pkg/tchart_web.js';
await init();
const svg = render_tcml("Clock _~_~_~");
```

## Documentation

- [TCML format](docs/spec/tcml-format.md)
- [CLI](docs/spec/cli.md)
- [Web (WASM)](docs/spec/web.md)
- [Web editor](docs/spec/editor.md)
- [WaveDrom export](docs/spec/wavedrom.md)

## Acknowledgements

This implementation is built on top of the following prior work, with gratitude:

- [Original tchart](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/indexframe.html) (Tohoku Gakuin University)
- [tchart-coffee](https://dora.bk.tsukuba.ac.jp/~takeuchi/?%E3%82%BD%E3%83%95%E3%83%88%E3%82%A6%E3%82%A7%E3%82%A2%2F%E3%82%BF%E3%82%A4%E3%83%9F%E3%83%B3%E3%82%B0%E3%83%81%E3%83%A3%E3%83%BC%E3%83%88%E6%B8%85%E6%9B%B8%E3%82%B5%E3%83%BC%E3%83%93%E3%82%B9) (University of Tsukuba)

## See also

- [WaveDrom](https://wavedrom.com/) — for high-quality, datasheet-grade timing diagrams
