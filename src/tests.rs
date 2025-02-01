#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::{converter, tex2typst, tex_parser, text_and_tex2typst, typst_writer};

    #[test]
    fn simple_test() {
        let test_list = vec![
            ("\\frac{1}{2}", "1 / 2"),
            ("\\sqrt{2}", "sqrt(2)"),
            ("\\sum_{i=1}^n i", "sum_(i = 1)^n i"),
            ("\\int_{a}^{b} f(x) dx", "integral_a^b f(x) d x"),
            ("\\sqrt[3]{x}", "root(3, x)"),
            ("e_f(x)", "e_f (x)"),
            ("e_{f (x)}", "e_(f(x))"),
        ];
        for (tex, typst) in test_list {
            assert_eq!(tex2typst(tex), typst);
        }
    }

    #[test]
    fn test_symbols() {
        let test_list = vec![("\\square", "square")];
        for (tex, typst) in test_list {
            assert_eq!(tex2typst(tex), typst);
        }
    }

    #[test]
    fn test_frac() {
        let tex = r"\frac{1}{2}";
        let custom_macros = HashMap::new();
        let tex_tree = tex_parser::parse_tex(tex, &custom_macros);
        let typst_tree = converter::convert_tree(&tex_tree);
        let mut writer = typst_writer::TypstWriter::new();
        writer.serialize(&typst_tree);
        let typst = writer.finalize();
        println!("{}", typst);
    }

    #[test]
    fn test_something() {
        let tex = r"Exercise 1.1.16. Let \(d_1, d_2 \geq 1\), and let \(E_1 \subset \mathbf{R}^{d_1}, E_2 \subset \mathbf{R}^{d_2}\) be Jordan measurable sets. Show that \(E_1 \times E_2 \subset \mathbf{R}^{d_1+d_2}\) is Jordan measurable, and \(m^{d_1+d_2}\left(E_1 \times E_2\right)=m^{d_1}\left(E_1\right) \times m^{d_2}\left(E_2\right)\).";
        println!("{}", text_and_tex2typst(tex));
    }

    #[test]
    fn test_smth() {
        let tex = r"\mathbf{R}^{d_1}";
        let custom_macros = HashMap::new();
        let tex_tree = tex_parser::parse_tex(tex, &custom_macros);
        let typst_tree = converter::convert_tree(&tex_tree);
        let mut writer = typst_writer::TypstWriter::new();
        writer.serialize(&typst_tree);
        let typst = writer.finalize();
        println!("{}", typst);
    }
}
