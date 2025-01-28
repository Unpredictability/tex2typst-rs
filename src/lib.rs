use std::collections::HashMap;

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
    let mut writer = typst_writer::TypstWriter::new(false, true);
    writer.serialize(&typst_tree);
    let typst = writer.finalize();
    typst
}
