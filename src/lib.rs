use regex::{Captures, Regex};
use std::collections::HashMap;

mod converter;
mod definitions;
mod map;
mod tests;
mod tex_parser;
mod typst_writer;

/// Converts a given TeX string to a Typst string.
///
/// This function takes a TeX string as input, parses it into a TeX tree,
/// converts the TeX tree to a Typst tree, serializes the Typst tree, and
/// returns the resulting Typst string.
///
/// # Arguments
///
/// * `tex` - A string slice that holds the TeX input.
///
/// # Returns
///
/// * `Result<String, String>` - On success, returns the Typst string wrapped in `Ok`.
///   On failure, returns an error message wrapped in `Err`.
///
/// # Errors
///
/// This function will return an error if the TeX parsing, conversion, or
/// serialization fails.
///
/// # Example
///
/// ```
/// use tex2typst_rs::tex2typst;
/// let tex_input = r"\frac{a}{b}";
/// let typst_output = tex2typst(tex_input).unwrap();
/// println!("{}", typst_output);
/// ```
pub fn tex2typst(tex: &str) -> Result<String, String> {
    let custom_macros = HashMap::new();
    let tex_tree = tex_parser::parse_tex(tex, &custom_macros)?;
    let typst_tree = converter::convert_tree(&tex_tree)?;
    let mut writer = typst_writer::TypstWriter::new();
    writer.serialize(&typst_tree)?;
    let typst = writer.finalize()?;
    Ok(typst)
}

/// Converts a given input string containing TeX math expressions to Typst format.
///
/// This function searches for inline and display math expressions within the input string,
/// converts them to Typst format using the `tex2typst` function, and returns the resulting string.
///
/// # Arguments
///
/// * `input` - A string slice that holds the input text containing TeX math expressions.
///
/// # Returns
///
/// * `Result<String, String>` - On success, returns the converted string wrapped in `Ok`.
///   On failure, returns an error message wrapped in `Err`.
///
/// # Errors
///
/// This function will return an error if the TeX to Typst conversion fails.
///
/// # Example
///
/// ```
/// use tex2typst_rs::text_and_tex2typst;
/// let input = r"This is inline math: \(a + b\) and this is display math: \[a^2 + b^2 = c^2\]";
/// let output = text_and_tex2typst(input).unwrap();
/// println!("{}", output);
/// ```
pub fn text_and_tex2typst(input: &str) -> Result<String, String> {
    let regex = Regex::new(r"\\\((.+?)\\\)|(?s)\\\[(.+?)\\\]").unwrap();

    replace_all(&regex, input, |caps: &Captures| {
        if let Some(inline_math) = caps.get(1) {
            let typst_math = tex2typst(inline_math.as_str().trim())?;
            Ok(format!("${}$", typst_math))
        } else if let Some(display_math) = caps.get(2) {
            let typst_math = tex2typst(display_math.as_str().trim())
                .map_err(|e| e.to_string())
                .unwrap();
            Ok(format!("$\n{}\n$", typst_math))
        } else {
            Ok(caps[0].to_string())
        }
    })
}

fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    replacement: impl Fn(&Captures) -> Result<String, E>,
) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
}
