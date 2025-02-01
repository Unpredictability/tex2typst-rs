use std::collections::HashMap;
use regex::Regex;

mod converter;
mod definitions;
mod map;
mod tests;
mod tex_parser;
mod typst_writer;

pub fn tex2typst(tex: &str) -> String {
    let custom_macros = HashMap::new();
    let tex_tree = tex_parser::parse_tex(tex, &custom_macros);
    let typst_tree = converter::convert_tree(&tex_tree);
    let mut writer = typst_writer::TypstWriter::new();
    writer.serialize(&typst_tree);
    let typst = writer.finalize();
    typst
}

pub fn text_and_tex2typst(input: &str) -> String {
    let regex = Regex::new(r"\\\((.+?)\\\)|(?s)\\\[(.+?)\\\]").unwrap();

    let output = regex.replace_all(input, |caps: &regex::Captures| {
        if let Some(inline_math) = caps.get(1) {
            let typst_math = tex2typst(inline_math.as_str().trim());
            format!("${}$", typst_math)
        } else if let Some(display_math) = caps.get(2) {
            let typst_math = tex2typst(display_math.as_str().trim());
            format!("$\n{}\n$", typst_math)
        } else {
            caps[0].to_string()
        }
    });

    output.to_string()
}
