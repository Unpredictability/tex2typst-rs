#![allow(unused_imports)]

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::error::Error;
    use std::result;
    use crate::{converter, tex2typst, tex2typst_with_macros, tex_parser, text_and_tex2typst, typst_writer};

    #[test]
    fn simple_test() {
        let test_list = vec![
            ("\\frac{1}{2}", "1/2"),
            ("\\sqrt{2}", "sqrt(2)"),
            ("\\sum_{i=1}^n i", "sum_(i = 1)^n i"),
            ("\\int_{a}^{b} f(x) dx", "integral_a^b f(x) d x"),
            ("\\sqrt[3]{x}", "root(3, x)"),
            ("e_f(x)", "e_f (x)"),
            ("e_{f (x)}", "e_(f(x))"),
        ];
        for (tex, typst) in test_list {
            assert_eq!(tex2typst(tex).unwrap(), typst);
        }
    }

    #[test]
    fn test_symbols() {
        let test_list = vec![("\\square", "square")];
        for (tex, typst) in test_list {
            assert_eq!(tex2typst(tex).unwrap(), typst);
        }
    }

    #[test]
    fn test_something() {
        let tex = r"Exercise 1.1.16. Let \(d_1, d_2 \geq 1\), and let \(E_1 \subset \mathbf{R}^{d_1}, E_2 \subset \mathbf{R}^{d_2}\) be Jordan measurable sets. Show that \(E_1 \times E_2 \subset \mathbf{R}^{d_1+d_2}\) is Jordan measurable, and \(m^{d_1+d_2}\left(E_1 \times E_2\right)=m^{d_1}\left(E_1\right) \times m^{d_2}\left(E_2\right)\).";
        println!("{}", text_and_tex2typst(tex).unwrap());
    }

    #[test]
    fn test_smth() -> Result<(), String> {
        let tex = r"\[\sqrt[a]{123} \frac{a\frac{a}{b}}{b} \frac{a}{b} !@#@$#%\]";
        let typst = text_and_tex2typst(tex).unwrap_or_else(|e| format!("Error: {}", e));
        println!("{}", &typst);
        Ok(())
    }

    #[test]
    fn test_macros() {
        let tex = r"\d^2";
        let mut custom_macros = HashMap::new();
        custom_macros.insert(r"\d".to_string(), r"\partial".to_string());
        let result = tex2typst_with_macros(tex, &custom_macros).unwrap_or_else(|e| format!("Error: {}", e));
        println!("{}", result);
    }

    #[test]
    fn test_floor() {
        let tex = r"\left\lfloor \frac{a}{b} \right\rfloor \floor{\frac{a}{b}}";
        let result = tex2typst(tex).unwrap();
        println!("{}", result);
    }

    #[test]
    fn test_text_with_space() {
        let tex = r"\text {some text}";
        let result = tex2typst(tex).unwrap();
        assert_eq!(result, "\"some text\"");

        let wrong_tex = r"\text ";
        let result = tex2typst(wrong_tex);
        assert!(result.is_err());
    }

    #[test]
    fn test_readme() {
        let tex = r"\widehat{f}(\xi)=\int_{-\infty}^{\infty} f(x) e^{-i 2 \pi \xi x} d x, \quad \forall \xi \in \mathbb{R}";
        println!("{}", tex2typst(tex).unwrap());

        let mixed = r"some text and some formula: \(\frac{1}{2}\)";
        println!("{}", text_and_tex2typst(mixed).unwrap());
    }
}
