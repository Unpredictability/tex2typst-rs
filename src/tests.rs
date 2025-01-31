#[cfg(test)]
mod tests {
    use crate::{tex2typst, text_and_tex2typst};

    #[test]
    fn simple_test() {
        let test_list = vec![
            ("\\frac{1}{2}", "frac(1, 2)"),
            ("\\sqrt{2}", "sqrt(2)"),
            ("\\sum_{i=1}^n i", "sum_(i = 1)^n i"),
            ("\\int_{a}^{b} f(x) dx", "integral_a^b f(x) d x"),
            ("\\sqrt[3]{x}", "root(3, x)"),
            ("e_f(x)", "e_f(x)"),
            ("e_{f (x)}", "e_(f(x))"),
        ];
        for (tex, typst) in test_list {
            assert_eq!(tex2typst(tex), typst);
        }
    }

    #[test]
    fn test_text_and_tex2typst() {
        let test_list = vec![
            ("some text and some formula: \\(\\frac{1}{2}\\)", "some text and some formula: $frac(1, 2)$"),
            ("Some text and a display math: \n\\[\n a^2 + b^2 = c^2\n\\]", "Some text and a display math: \n$\na^2 + b^2 = c^2\n$")
        ];
        for (text_and_tex, typst) in test_list {
            assert_eq!(text_and_tex2typst(text_and_tex), typst);
        }
    }
}
