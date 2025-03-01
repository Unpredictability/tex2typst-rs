use crate::definitions::{TexNode, TexNodeData, TexNodeType, TypstNode, TypstNodeData, TypstNodeType, TypstSupsubData};
use crate::map::SYMBOL_MAP;
use std::collections::HashMap;

// Symbols that are supported by Typst but not by KaTeX
const TYPST_INTRINSIC_SYMBOLS: &[&str] = &[
    "dim", "id", "im", "mod", "Pr", "sech", "csch",
    // "sgn"
];

pub fn convert_tree(node: &TexNode) -> Result<TypstNode, String> {
    match node.node_type {
        TexNodeType::Empty => Ok(TypstNode::new(TypstNodeType::Empty, String::from(""), None, None)),
        TexNodeType::Whitespace => Ok(TypstNode::new(
            TypstNodeType::Whitespace,
            node.content.clone(),
            None,
            None,
        )),
        TexNodeType::NoBreakSpace => Ok(TypstNode::new(
            TypstNodeType::NoBreakSpace,
            node.content.clone(),
            None,
            None,
        )),
        TexNodeType::Ordgroup => Ok(TypstNode::new(
            TypstNodeType::Group,
            String::from(""),
            Some(
                node.args
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|arg| convert_tree(arg))
                    .collect::<Result<Vec<_>, String>>()?,
            ),
            None,
        )),
        TexNodeType::Element => Ok(TypstNode::new(
            TypstNodeType::Atom,
            convert_token(&node.content),
            None,
            None,
        )),
        TexNodeType::Symbol => Ok(TypstNode::new(
            TypstNodeType::Symbol,
            convert_token(&node.content),
            None,
            None,
        )),
        TexNodeType::Text => Ok(TypstNode::new(TypstNodeType::Text, node.content.clone(), None, None)),
        TexNodeType::Comment => Ok(TypstNode::new(TypstNodeType::Comment, node.content.clone(), None, None)),
        TexNodeType::SupSub => {
            let TexNodeData::Supsub(data) = node.data.as_ref().unwrap().as_ref() else {
                return Err("SupSub node does not have data".to_string());
            };
            let base = &data.base;
            let sup = data.sup.as_ref();
            let sub = data.sub.as_ref();

            // Special logic for overbrace
            if base.node_type == TexNodeType::UnaryFunc && base.content == "\\overbrace" && sup.is_some() {
                return Ok(TypstNode::new(
                    TypstNodeType::FuncCall,
                    "overbrace".to_string(),
                    Some(vec![
                        convert_tree(&base.args.as_ref().unwrap()[0])?,
                        convert_tree(sup.unwrap())?,
                    ]),
                    None,
                ));
            } else if base.node_type == TexNodeType::UnaryFunc && base.content == "\\underbrace" && sub.is_some() {
                return Ok(TypstNode::new(
                    TypstNodeType::FuncCall,
                    "underbrace".to_string(),
                    Some(vec![
                        convert_tree(&base.args.as_ref().unwrap()[0])?,
                        convert_tree(sub.unwrap())?,
                    ]),
                    None,
                ));
            }

            let mut typst_data = TypstSupsubData {
                base: convert_tree(base)?,
                sup: None,
                sub: None,
            };

            if typst_data.base.node_type == TypstNodeType::Empty {
                typst_data.base = TypstNode::new(TypstNodeType::Text, "".to_string(), None, None);
            }
            if let Some(sup) = sup {
                typst_data.sup = Some(convert_tree(sup)?);
            }
            if let Some(sub) = sub {
                typst_data.sub = Some(convert_tree(sub)?);
            }

            Ok(TypstNode::new(
                TypstNodeType::Supsub,
                "".to_string(),
                None,
                Some(Box::from(TypstNodeData::Supsub(typst_data))),
            ))
        }
        TexNodeType::Leftright => {
            let args = node.args.as_ref().unwrap();
            let left = &args[0];
            let right = &args[2];
            let mut group = TypstNode::new(
                TypstNodeType::Group,
                "".to_string(),
                Some(
                    args.iter()
                        .map(|arg| convert_tree(arg))
                        .collect::<Result<Vec<_>, String>>()?,
                ),
                None,
            );
            if matches!(
                (left.content.as_str(), right.content.as_str()),
                ("[", "]")
                    | ("(", ")")
                    | ("\\{", "\\}")
                    | ("\\lfloor", "\\rfloor")
                    | ("\\lceil", "\\rceil")
                    | ("\\lfloor", "\\rceil")
            ) {
                return Ok(group);
            }

            if right.content == "." {
                group.args.as_mut().unwrap().pop();
                return Ok(group);
            } else if left.content == "." {
                group.args.as_mut().unwrap().remove(0);
                return Ok(TypstNode::new(
                    TypstNodeType::FuncCall,
                    "lr".to_string(),
                    Some(vec![group]),
                    None,
                ));
            }
            Ok(TypstNode::new(
                TypstNodeType::FuncCall,
                "lr".to_string(),
                Some(vec![group]),
                None,
            ))
        }
        TexNodeType::OptionBinaryFunc => {
            if node.content == "\\sqrt" {
                match node.args.as_ref().unwrap().len() {
                    1 => {
                        let mandatory_arg = convert_tree(&node.args.as_ref().unwrap()[0])?;
                        Ok(TypstNode::new(
                            TypstNodeType::FuncCall,
                            "sqrt".to_string(),
                            Some(vec![mandatory_arg]),
                            None,
                        ))
                    }
                    2 => {
                        let optional_arg = convert_tree(&node.args.as_ref().unwrap()[0])?;
                        let mandatory_arg = convert_tree(&node.args.as_ref().unwrap()[1])?;
                        Ok(TypstNode::new(
                            TypstNodeType::FuncCall,
                            "root".to_string(),
                            Some(vec![optional_arg, mandatory_arg]),
                            None,
                        ))
                    }
                    _ => Err(format!(
                        "Invalid number of arguments for \\sqrt: {}",
                        node.args.as_ref().unwrap().len()
                    )),
                }
            } else {
                Err(format!("Unknown option binary function: {}", node.content))
            }
        }
        TexNodeType::BinaryFunc => {
            if node.content == "\\overset" {
                return convert_overset(node);
            }

            // \frac{a}{b} -> a / b
            if node.content == "\\frac" {
                let args = node.args.as_ref().unwrap();
                let num = convert_tree(&args[0])?;
                let den = convert_tree(&args[1])?;
                return Ok(TypstNode::new(
                    TypstNodeType::Fraction,
                    "".to_string(),
                    Some(vec![num, den]),
                    None,
                ));
            }

            Ok(TypstNode::new(
                TypstNodeType::FuncCall,
                convert_token(&node.content),
                Some(
                    node.args
                        .as_ref()
                        .ok_or("Binary function node does not have args")?
                        .iter()
                        .map(|arg| convert_tree(arg))
                        .collect::<Result<Vec<_>, String>>()?,
                ),
                None,
            ))
        }
        TexNodeType::UnaryFunc => {
            let arg0 = convert_tree(&node.args.as_ref().unwrap()[0])?;
            if node.content == "\\mathbf" {
                let inner = TypstNode::new(TypstNodeType::FuncCall, "bold".to_string(), Some(vec![arg0]), None);
                return Ok(TypstNode::new(
                    TypstNodeType::FuncCall,
                    "upright".to_string(),
                    Some(vec![inner]),
                    None,
                ));
            }
            if node.content == "\\mathbb"
                && arg0.node_type == TypstNodeType::Atom
                && arg0.content.chars().all(|c| c.is_ascii_uppercase())
            {
                return Ok(TypstNode::new(
                    TypstNodeType::Symbol,
                    arg0.content.repeat(2),
                    None,
                    None,
                ));
            }
            if node.content == "\\operatorname" {
                let body = node.args.as_ref().unwrap();
                if body.len() != 1 || body[0].node_type != TexNodeType::Text {
                    return Err(format!(
                        "Expecting body of \\operatorname to be text but got {:?}",
                        node
                    ));
                }
                let text = &body[0].content;
                return if TYPST_INTRINSIC_SYMBOLS.contains(&text.as_str()) {
                    Ok(TypstNode::new(TypstNodeType::Symbol, text.to_string(), None, None))
                } else {
                    Ok(TypstNode::new(
                        TypstNodeType::FuncCall,
                        "op".to_string(),
                        Some(vec![TypstNode::new(TypstNodeType::Text, text.to_string(), None, None)]),
                        None,
                    ))
                };
            }
            Ok(TypstNode::new(
                TypstNodeType::FuncCall,
                convert_token(&node.content),
                Some(
                    node.args
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|arg| convert_tree(arg))
                        .collect::<Result<Vec<_>, String>>()?,
                ),
                None,
            ))
        }
        TexNodeType::BeginEnd => {
            let TexNodeData::Array(matrix) = node.data.as_ref().unwrap().as_ref() else {
                panic!()
            };
            let data: Vec<Vec<TypstNode>> = matrix
                .iter()
                .map(|row| row.iter().map(|n| convert_tree(n)).collect::<Result<Vec<_>, String>>())
                .collect::<Result<_, String>>()?;
            if node.content.starts_with("align") {
                Ok(TypstNode::new(
                    TypstNodeType::Align,
                    "".to_string(),
                    None,
                    Some(Box::from(TypstNodeData::Array(data))),
                ))
            } else {
                let mut res = TypstNode::new(
                    TypstNodeType::Matrix,
                    "".to_string(),
                    None,
                    Some(Box::from(TypstNodeData::Array(data))),
                );
                res.set_options(HashMap::from([("delim".to_string(), "#none".to_string())]));
                Ok(res)
            }
        }
        TexNodeType::UnknownMacro => Ok(TypstNode::new(
            TypstNodeType::Unknown,
            convert_token(&node.content),
            None,
            None,
        )),
        TexNodeType::Control => {
            if node.content == "\\\\" {
                Ok(TypstNode::new(TypstNodeType::Symbol, "\\".to_string(), None, None))
            } else if node.content == "\\," {
                Ok(TypstNode::new(TypstNodeType::Symbol, "thin".to_string(), None, None))
            } else {
                return Err(format!("Unknown control sequence: {:?}", node));
            }
        }
        TexNodeType::Unknown => Ok(TypstNode::new(
            TypstNodeType::Unknown,
            convert_token(&node.content),
            None,
            None,
        )),
    }
}

fn convert_token(token: &str) -> String {
    if token.chars().all(|c| c.is_alphanumeric()) {
        token.to_string()
    } else if token == "/" {
        "\\/".to_string()
    } else if token == "\\|" {
        // \| in LaTeX is double vertical bar looks like ||
        "parallel".to_string()
    } else if token == "\\\\" {
        "\\".to_string()
    } else if ["\\$", "\\#", "\\&", "\\_"].contains(&token) {
        token.to_string()
    } else if token.starts_with('\\') {
        let symbol = &token[1..];
        if let Some(mapped_symbol) = SYMBOL_MAP.get(symbol) {
            mapped_symbol.to_string()
        } else {
            // Fall back to the original macro.
            // This works for \alpha, \beta, \gamma, etc.
            // If this.nonStrict is true, this also works for all unknown macros.
            symbol.to_string()
        }
    } else {
        token.to_string()
    }
}

fn convert_overset(node: &TexNode) -> Result<TypstNode, String> {
    let args = node.args.as_ref().unwrap();
    let sup = &args[0];
    let base = &args[1];

    let is_def = |n: &TexNode| -> bool {
        if n.eq(&TexNode::new(TexNodeType::Text, "def".to_string(), None, None)) {
            return true;
        }
        if n.node_type == TexNodeType::Ordgroup && n.args.as_ref().unwrap().len() == 3 {
            let args = n.args.as_ref().unwrap();
            let d = TexNode::new(TexNodeType::Element, "d".to_string(), None, None);
            let e = TexNode::new(TexNodeType::Element, "e".to_string(), None, None);
            let f = TexNode::new(TexNodeType::Element, "f".to_string(), None, None);
            return args[0].eq(&d) && args[1].eq(&e) && args[2].eq(&f);
        }
        false
    };

    let is_eq = |n: &TexNode| -> bool { n.eq(&TexNode::new(TexNodeType::Element, "=".to_string(), None, None)) };

    if is_def(sup) && is_eq(base) {
        return Ok(TypstNode::new(TypstNodeType::Symbol, "eq.def".to_string(), None, None));
    }

    let mut op_call = TypstNode::new(
        TypstNodeType::FuncCall,
        "op".to_string(),
        Some(vec![convert_tree(base)?]),
        None,
    );
    op_call.set_options(HashMap::from([("limits".to_string(), "true".to_string())]));

    Ok(TypstNode::new(
        TypstNodeType::Supsub,
        "".to_string(),
        None,
        Some(Box::from(TypstNodeData::Supsub(TypstSupsubData {
            base: op_call,
            sup: Some(convert_tree(sup)?),
            sub: None,
        }))),
    ))
}
