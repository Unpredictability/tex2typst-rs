#![allow(unused_imports)]

#[cfg(test)]
mod tests {
    use crate::{converter, tex2typst, tex2typst_with_macros, tex_parser, text_and_tex2typst, typst_writer};
    use std::collections::HashMap;
    use std::error::Error;
    use std::result;

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
    fn test_invalid_input() -> Result<(), String> {
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
        assert_eq!(result, "diff^2");
    }

    #[test]
    fn test_floor() {
        let tex = r"\left\lfloor \frac{a}{b} \right\rfloor \floor{\frac{a}{b}}";
        let result = tex2typst(tex).unwrap();
        assert_eq!(result, "floor(a/b) floor(a/b)");
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
    fn test_cn_chars() {
        let tex = r"\begin{aligned}asd \end{aligned}";
        let result = tex2typst(tex).unwrap();
        assert_eq!(result, "a s d");
    }

    #[test]
    fn test_lacking_space() {
        let tex = r"           x       =  \frac{a-b \pm \sqrt{b^2 - 4ac}}{2a} ";
        let result = tex2typst(tex).unwrap();
        assert_eq!(result, "x = (a - b plus.minus sqrt(b^2 - 4 a c))/(2 a)");
    }

    #[test]
    fn test_readme() {
        let tex =
            r"\widehat{f}(\xi)=\int_{-\infty}^{\infty} f(x) e^{-i 2 \pi \xi x} d x, \quad \forall \xi \in \mathbb{R}";
        println!("{}", tex2typst(tex).unwrap());

        let mixed = r"some text and some formula: \(\frac{1}{2}\)";
        println!("{}", text_and_tex2typst(mixed).unwrap());
    }
}
