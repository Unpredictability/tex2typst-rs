use crate::command_registry::{CommandRegistry, CommandType};
use crate::definitions::TexNodeData::Array;
use crate::definitions::{TexNode, TexNodeData, TexNodeType, TexSupsubData, TexToken, TexTokenType};
use crate::map::SYMBOL_MAP;
use crate::tex_parser_utils::*;
use crate::tex_tokenizer;
use std::cmp::PartialEq;

type ParseResult = Result<(TexNode, usize), String>;

pub struct LatexParser {
    space_sensitive: bool,
    newline_sensitive: bool,
    command_registry: CommandRegistry,
}

impl LatexParser {
    pub fn new(space_sensitive: bool, newline_sensitive: bool) -> Self {
        LatexParser {
            space_sensitive,
            newline_sensitive,
            command_registry: CommandRegistry::new(),
        }
    }

    pub fn parse(&self, tokens: Vec<TexToken>) -> Result<TexNode, String> {
        let mut results: Vec<TexNode> = Vec::new();
        let mut pos = 0;

        while pos < tokens.len() {
            let (res, new_pos) = self.parse_next_expr(&tokens, pos)?;
            pos = new_pos;
            if res.node_type == TexNodeType::Whitespace
                && (!self.space_sensitive && res.content.replace(" ", "").is_empty()
                    || !self.newline_sensitive && res.content == "\n")
            {
                continue;
            }
            if res.node_type == TexNodeType::Control && res.content == "&" {
                return Err("Unexpected & outside of an alignment".to_string());
            } else {
                results.push(res);
            }
        }

        if results.is_empty() {
            Ok(EMPTY_NODE.clone())
        } else if results.len() == 1 {
            Ok(results.remove(0))
        } else {
            Ok(TexNode::new(TexNodeType::Ordgroup, String::new(), Some(results), None))
        }
    }

    fn parse_next_expr(&self, tokens: &[TexToken], start: usize) -> ParseResult {
        let (base, mut pos) = self.parse_next_expr_without_supsub(tokens, start)?;
        let mut sub: Option<TexNode> = None;
        let mut sup: Option<TexNode> = None;
        let mut num_prime = 0;

        num_prime += eat_primes(tokens, pos);
        pos += num_prime;
        if pos < tokens.len() && tokens[pos] == *SUB_SYMBOL {
            let (sub_node, new_pos) = self.parse_next_expr_without_supsub(tokens, pos + 1)?;
            sub = Some(sub_node);
            pos = new_pos;
            num_prime += eat_primes(tokens, pos);
            pos += num_prime;
            if pos < tokens.len() && tokens[pos] == *SUP_SYMBOL {
                let (sup_node, new_pos) = self.parse_next_expr_without_supsub(tokens, pos + 1)?;
                sup = Some(sup_node);
                pos = new_pos;
                if eat_primes(tokens, pos) > 0 {
                    panic!("Double superscript");
                }
            }
        } else if pos < tokens.len() && tokens[pos] == *SUP_SYMBOL {
            let (sup_node, new_pos) = self.parse_next_expr_without_supsub(tokens, pos + 1)?;
            sup = Some(sup_node);
            pos = new_pos;
            if eat_primes(tokens, pos) > 0 {
                panic!("Double superscript");
            }
            if pos < tokens.len() && tokens[pos] == *SUB_SYMBOL {
                let (sub_node, new_pos) = self.parse_next_expr_without_supsub(tokens, pos + 1)?;
                sub = Some(sub_node);
                pos = new_pos;
                if eat_primes(tokens, pos) > 0 {
                    panic!("Double superscript");
                }
            }
        }

        if sub.is_some() || sup.is_some() || num_prime > 0 {
            let mut res = TexSupsubData {
                base,
                sub: None,
                sup: None,
            };
            if let Some(sub_node) = sub {
                res.sub = Some(sub_node);
            }
            if num_prime > 0 {
                let mut sup_node = TexNode::new(TexNodeType::Ordgroup, String::new(), Some(Vec::new()), None);
                for _ in 0..num_prime {
                    sup_node.args.as_mut().unwrap().push(TexNode::new(
                        TexNodeType::Element,
                        "'".to_string(),
                        None,
                        None,
                    ));
                }
                if let Some(sup_node_inner) = sup {
                    sup_node.args.as_mut().unwrap().push(sup_node_inner);
                }
                if sup_node.args.as_ref().unwrap().len() == 1 {
                    res.sup = Some(sup_node.args.unwrap().remove(0));
                } else {
                    res.sup = Some(sup_node);
                }
            } else if let Some(sup_node) = sup {
                res.sup = Some(sup_node);
            }
            Ok((
                TexNode::new(
                    TexNodeType::SupSub,
                    String::new(),
                    None,
                    Some(Box::from(TexNodeData::Supsub(res))),
                ),
                pos,
            ))
        } else {
            Ok((base, pos))
        }
    }

    fn parse_next_expr_without_supsub(&self, tokens: &[TexToken], start: usize) -> ParseResult {
        match tokens.get(start) {
            None => Err("Unexpected end of input".to_string()),
            Some(_first_token) => {
                let first_token = _first_token;
                let token_type = &first_token.token_type;
                match token_type {
                    TexTokenType::Element => Ok((
                        TexNode::new(TexNodeType::Element, first_token.value.clone(), None, None),
                        start + 1,
                    )),
                    TexTokenType::Text => Ok((
                        TexNode::new(TexNodeType::Text, first_token.value.clone(), None, None),
                        start + 1,
                    )),
                    TexTokenType::Comment => Ok((
                        TexNode::new(TexNodeType::Comment, first_token.value.clone(), None, None),
                        start + 1,
                    )),
                    TexTokenType::Space | TexTokenType::Newline => Ok((
                        TexNode::new(TexNodeType::Whitespace, first_token.value.clone(), None, None),
                        start + 1,
                    )),
                    TexTokenType::NoBreakSpace => Ok((
                        TexNode::new(TexNodeType::NoBreakSpace, first_token.value.clone(), None, None),
                        start + 1,
                    )),
                    TexTokenType::Command => {
                        if first_token.eq(&BEGIN_COMMAND) {
                            self.parse_begin_end_expr(tokens, start)
                        } else if first_token.eq(&LEFT_COMMAND) {
                            self.parse_left_right_expr(tokens, start)
                        } else {
                            self.parse_command_expr(tokens, start)
                        }
                    }
                    TexTokenType::Control => {
                        let control_char = &first_token.value;
                        match control_char.as_str() {
                            "{" => {
                                let pos_closing_bracket =
                                    find_closing_match(tokens, start, &LEFT_CURLY_BRACKET, &RIGHT_CURLY_BRACKET);
                                if pos_closing_bracket == -1 {
                                    Err("Unmatched '{'".to_string())
                                } else {
                                    let expr_inside = &tokens[start + 1..pos_closing_bracket as usize];
                                    Ok((self.parse(expr_inside.to_vec())?, pos_closing_bracket as usize + 1))
                                }
                            }
                            "}" => Err("Unexpected '}'".to_string()),
                            "\\\\" => Ok((
                                TexNode::new(TexNodeType::Control, "\\\\".to_string(), None, None),
                                start + 1,
                            )),
                            "\\," => Ok((
                                TexNode::new(TexNodeType::Control, "\\,".to_string(), None, None),
                                start + 1,
                            )),
                            "_" | "^" => Ok((EMPTY_NODE.clone(), start)),
                            "&" => Ok((
                                TexNode::new(TexNodeType::Control, "&".to_string(), None, None),
                                start + 1,
                            )),
                            _ => Err("Unknown control sequence".to_string()),
                        }
                    }
                    TexTokenType::Unknown => Ok((
                        TexNode::new(TexNodeType::Unknown, first_token.value.clone(), None, None),
                        start + 1,
                    )),
                }
            }
        }
    }

    fn parse_command_expr(&self, tokens: &[TexToken], start: usize) -> ParseResult {
        let command = &tokens[start].value; // command name starts with a \\
        let pos = start + 1;

        if matches!(command[1..].as_ref(), "left" | "right" | "begin" | "end") {
            return Err(format!("Unexpected command: {}", command));
        }

        match self.command_registry.get_command_type(&command[1..]) {
            Some(CommandType::Symbol) => {
                if !SYMBOL_MAP.contains_key(&command[1..]) {
                    return Ok((
                        TexNode::new(TexNodeType::UnknownMacro, command.clone(), None, None),
                        pos,
                    ));
                }
                Ok((TexNode::new(TexNodeType::Symbol, command.clone(), None, None), pos))
            }
            Some(CommandType::Unary) => {
                if pos >= tokens.len() {
                    return Err(format!("Expecting argument for {}", command));
                }
                if command == "\\text" {
                    if pos + 2 >= tokens.len() {
                        return Err("Expecting content for \\text command".to_string());
                    }
                    assert_eq!(tokens[pos], *LEFT_CURLY_BRACKET);
                    assert_eq!(tokens[pos + 1].token_type, TexTokenType::Text);
                    assert_eq!(tokens[pos + 2], *RIGHT_CURLY_BRACKET);
                    let text = tokens[pos + 1].value.clone();
                    return Ok((TexNode::new(TexNodeType::Text, text, None, None), pos + 3));
                }
                let (arg1, new_pos) = self.parse_next_expr_without_supsub(tokens, pos)?;
                Ok((
                    TexNode::new(TexNodeType::UnaryFunc, command.clone(), Some(vec![arg1]), None),
                    new_pos,
                ))
            }
            Some(CommandType::Binary) => {
                let (arg1, pos1) = self.parse_next_expr_without_supsub(tokens, pos)?;
                let (arg2, pos2) = self.parse_next_expr_without_supsub(tokens, pos1)?;
                Ok((
                    TexNode::new(TexNodeType::BinaryFunc, command.clone(), Some(vec![arg1, arg2]), None),
                    pos2,
                ))
            }
            Some(CommandType::OptionalBinary) => {
                let mut args = vec![];
                let mut new_pos = pos;
                if tokens[pos].token_type == TexTokenType::Element && tokens[pos].value == "[" {
                    let pos_left_square_bracket = pos;
                    let pos_right_square_bracket =
                        find_closing_match(tokens, pos, &LEFT_SQUARE_BRACKET, &RIGHT_SQUARE_BRACKET);
                    if pos_right_square_bracket == -1 {
                        return Err("No matching right square bracket for [".to_string());
                    }
                    let optional_arg_inside = &tokens[pos_left_square_bracket + 1..pos_right_square_bracket as usize];
                    let optional_arg_node = self.parse(optional_arg_inside.to_vec())?;
                    let (mandatory_arg_node, _new_pos) =
                        self.parse_next_expr_without_supsub(tokens, pos_right_square_bracket as usize + 1)?;
                    args.push(optional_arg_node);
                    args.push(mandatory_arg_node);
                    new_pos = _new_pos;
                } else {
                    let (arg1, _new_pos) = self.parse_next_expr_without_supsub(tokens, pos)?;
                    args.push(arg1);
                    new_pos = _new_pos;
                }
                Ok((
                    TexNode::new(TexNodeType::OptionBinaryFunc, command.clone(), Some(args), None),
                    new_pos,
                ))
            }
            _ => Err("Invalid number of parameters".to_string()),
        }
    }

    fn parse_left_right_expr(&self, tokens: &[TexToken], start: usize) -> ParseResult {
        assert!(tokens[start].eq(&LEFT_COMMAND));

        let mut pos = start + 1;
        pos += eat_whitespaces(tokens, pos);

        if pos >= tokens.len() {
            return Err("Expecting delimiter after \\left".to_string());
        }

        let left_delimiter = eat_parenthesis(tokens, pos);
        if left_delimiter.is_none() {
            return Err("Invalid delimiter after \\left".to_string());
        }
        pos += 1;
        let expr_inside_start = pos;
        let idx = find_closing_right_command(tokens, start);
        if idx == -1 {
            return Err("No matching \\right".to_string());
        }
        let expr_inside_end = idx as usize;
        pos = expr_inside_end + 1;

        pos += eat_whitespaces(tokens, pos);
        if pos >= tokens.len() {
            return Err("Expecting \\right after \\left".to_string());
        }

        let right_delimiter = eat_parenthesis(tokens, pos);
        if right_delimiter.is_none() {
            return Err("Invalid delimiter after \\right".to_string());
        }
        pos += 1;

        let expr_inside = &tokens[expr_inside_start..expr_inside_end];
        let body = self.parse(expr_inside.to_vec())?;
        let args: Vec<TexNode> = vec![
            TexNode::new(TexNodeType::Element, left_delimiter.unwrap().value.clone(), None, None),
            body,
            TexNode::new(TexNodeType::Element, right_delimiter.unwrap().value.clone(), None, None),
        ];
        let res = TexNode::new(TexNodeType::Leftright, String::new(), Some(args), None);
        Ok((res, pos))
    }

    fn parse_begin_end_expr(&self, tokens: &[TexToken], start: usize) -> ParseResult {
        assert!(tokens[start].eq(&BEGIN_COMMAND));

        let mut pos = start + 1;
        assert!(tokens[pos].eq(&LEFT_CURLY_BRACKET));
        assert_eq!(tokens[pos + 1].token_type, TexTokenType::Text);
        assert!(tokens[pos + 2].eq(&RIGHT_CURLY_BRACKET));
        let env_name = tokens[pos + 1].value.clone();
        pos += 3;

        pos += eat_whitespaces(tokens, pos); // ignore whitespaces and '\n' after \begin{envName}

        let expr_inside_start = pos;

        let end_idx = find_closing_end_command(tokens, start);
        if end_idx == -1 {
            panic!("No matching \\end");
        }
        let expr_inside_end = end_idx as usize;
        pos = expr_inside_end + 1;

        assert!(tokens[pos].eq(&LEFT_CURLY_BRACKET));
        assert_eq!(tokens[pos + 1].token_type, TexTokenType::Text);
        assert!(tokens[pos + 2].eq(&RIGHT_CURLY_BRACKET));
        if tokens[pos + 1].value != env_name {
            return Err("Mismatched \\begin and \\end environments".to_string());
        }
        pos += 3;

        let mut expr_inside = tokens[expr_inside_start..expr_inside_end].to_vec();
        // ignore spaces and '\n' before \end{envName}
        while !expr_inside.is_empty()
            && matches!(
                expr_inside.last().unwrap().token_type,
                TexTokenType::Space | TexTokenType::Newline
            )
        {
            expr_inside.pop();
        }
        let body = self.parse_aligned(&*expr_inside)?;
        let res = TexNode::new(TexNodeType::BeginEnd, env_name, None, Some(Box::from(Array(body))));
        Ok((res, pos))
    }

    fn parse_aligned(&self, tokens: &[TexToken]) -> Result<Vec<Vec<TexNode>>, String> {
        let mut pos = 0;
        let mut all_rows: Vec<Vec<TexNode>> = vec![vec![TexNode::new(
            TexNodeType::Ordgroup,
            String::new(),
            Some(Vec::<TexNode>::new()),
            None,
        )]];
        let mut row: &mut Vec<TexNode> = &mut all_rows[0];
        let mut group: &mut TexNode = &mut row[0];

        while pos < tokens.len() {
            let (res, new_pos) = self.parse_next_expr(tokens, pos)?;
            pos = new_pos;

            if res.node_type == TexNodeType::Whitespace {
                if !self.space_sensitive && res.content.replace(" ", "").is_empty() {
                    continue;
                }
                if !self.newline_sensitive && res.content == "\n" {
                    continue;
                }
            }

            if res.node_type == TexNodeType::Control && res.content == "\\\\" {
                all_rows.push(vec![TexNode::new(
                    TexNodeType::Ordgroup,
                    String::new(),
                    Some(Vec::<TexNode>::new()),
                    None,
                )]);
                row = all_rows.last_mut().unwrap();
                group = &mut row[0];
            } else if res.node_type == TexNodeType::Control && res.content == "&" {
                row.push(TexNode::new(
                    TexNodeType::Ordgroup,
                    String::new(),
                    Some(Vec::new()),
                    None,
                ));
                group = row.last_mut().unwrap();
            } else {
                group.args.as_mut().unwrap().push(res);
            }
        }

        Ok(all_rows)
    }
}

fn pass_expand_custom_tex_macros(
    tokens: Vec<TexToken>,
    custom_tex_macros: &std::collections::HashMap<String, String>,
) -> Vec<TexToken> {
    let mut out_tokens: Vec<TexToken> = Vec::new();
    for token in tokens {
        if token.token_type == TexTokenType::Command {
            if let Some(expansion) = custom_tex_macros.get(&token.value) {
                if let Ok(expanded_tokens) = tex_tokenizer::tokenize(expansion) {
                    out_tokens.extend(expanded_tokens);
                }
            } else {
                out_tokens.push(token);
            }
        } else {
            out_tokens.push(token);
        }
    }
    out_tokens
}

pub fn parse_tex(tex: &str) -> Result<TexNode, String> {
    let parser = LatexParser::new(false, false);
    let tokens = tex_tokenizer::tokenize(tex)?;
    parser.parse(tokens)
}

