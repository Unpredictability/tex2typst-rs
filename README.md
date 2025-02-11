# tex2typst-rs

<a href="https://crates.io/crates/tex2typst-rs">
    <img alt="Crate" src="https://img.shields.io/crates/v/tex2typst-rs"
  ></a>
<a href="https://docs.rs/tex2typst-rs">
    <img alt="Documentation" src="https://docs.rs/tex2typst-rs/badge.svg"
  ></a>

A Rust library that converts TeX code to Typst code. 

Now you can try this library online in the [WASM web app (WIP)](https://unpredictability.github.io/tex2typst-UI/)!

# Aim of this project

There exist some other libraries that convert LaTeX (especially LaTeX math) to other languages. 
However, the result may not be visually pleasing or easy to read. 
This project aims to convert LaTeX to idiomatic Typst code, which can be very easily read and edited. 

For comparison, for this LaTeX input:

```latex
\overrightarrow{P M}=(3-x-y) \overrightarrow{P A}+x \overrightarrow{P B}+(y-2) \overrightarrow{P C}
```

[`mitex`](https://crates.io/crates/mitex) gives the output:

```typst
arrow(P  M )= \(3 - x - y \) arrow(P  A )+ x  arrow(P  B )+ \(y - 2 \) arrow(P  C )
```

`tex2typst-rs` gives the output:

```typst
arrow(P M) =(3 - x - y) arrow(P A) + x arrow(P B) +(y - 2) arrow(P C)
```

# Usage

See the [documentation](https://docs.rs/tex2typst-rs) for more details.

```Rust
use tex2typst_rs::tex2typst;
use tex2typst_rs::text_and_tex2typst;

fn main() {
    let tex = r"\widehat{f}(\xi)=\int_{-\infty}^{\infty} f(x) e^{-i 2 \pi \xi x} d x, \quad \forall \xi \in \mathbb{R}";
    println!("{}", tex2typst(tex).unwrap());
    
    let mixed = r"some text and some formula: \(\frac{1}{2}\)";
    println!("{}", text_and_tex2typst(mixed).unwrap());
}
```

Output:

```typst
hat(f)(xi) = int_(- infty)^infty f(x) e^(- i 2 pi xi x) d x, quad forall xi in RR
some text and some formula: $1/2$
```

# Acknowledgements

Took inspiration from [tex2typst](https://github.com/qwinsi/tex2typst).
