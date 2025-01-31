# tex2typst-rs
A Rust library that converts TeX code to Typst code.

Mainly took insipiration from [tex2typst](https://github.com/qwinsi/tex2typst).

# Usage

```Rust
use tex2typst_rs::tex2typst;

fn main() {
    let tex1 = "i_D = \\mu_n C_\\text{ox} \\frac{W}{L} \\left[ (v_\\text{GS} - V_t)v_\\text{DS} - \\frac{1}{2} v_\\text{DS}^2 \\right]";
    let tex2 = "\\iint_{\\Sigma} \\operatorname{curl}(\\vec{F}) \\cdot \\mathrm{d}\\vec{S} = \\oint_{\\partial \\Sigma} \\vec{F} \\times \\mathrm{d}\\vec{l}";
    println!("{}", tex2typst(tex1));
    println!("{}", tex2typst(tex2));
}
```

Output:

```
i_D = mu_n C_"ox" frac(W, L) [(v_"GS" - V_t ) v_"DS" - frac(1, 2) v_"DS"^2 ]
integral.double_Sigma op("curl")(arrow(F)) dot.op upright(d) arrow(S) = integral.cont_(diff Sigma) arrow(F) times upright(d) arrow(l)
```

Now it also supports mixed text and math mode with the new function `text_and_tex2typst`:

```Rust
use tex2typst_rs::text_and_tex2typst;

fn main() {
    let test_list = vec![
        ("some text and some formula: \\(\\frac{1}{2}\\)", "some text and some formula: $frac(1, 2)$"),
        ("Some text and a display math: \n\\[\n a^2 + b^2 = c^2\n\\]", "Some text and a display math: \n$\na^2 + b^2 = c^2\n$")
    ];
    for (text_and_tex, typst) in test_list {
        assert_eq!(text_and_tex2typst(text_and_tex), typst);
    }
}
```