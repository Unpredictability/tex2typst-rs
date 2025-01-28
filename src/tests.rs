#[cfg(test)]
mod tests {
    use crate::tex2typst;

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
}
