use crate::definitions::TexNodeData::{Array, Sqrt};
use crate::definitions::{TexNode, TexNodeData, TexNodeType, TexSupsubData, TexToken, TexTokenType};
use crate::map::get_symbol_map;
use std::cmp::PartialEq;
use std::sync::LazyLock;

const UNARY_COMMANDS: &[&str] = &[
    "sqrt",
    "text",
    "bar",
    "bold",
    "boldsymbol",
    "ddot",
    "dot",
    "hat",
    "mathbb",
    "mathbf",
    "mathcal",
    "mathfrak",
    "mathit",
    "mathrm",
    "mathscr",
    "mathsf",
    "mathtt",
    "operatorname",
    "overbrace",
    "overline",
    "pmb",
    "rm",
    "tilde",
    "underbrace",
    "underline",
    "vec",
    "overrightarrow",
    "widehat",
    "widetilde",
];

const BINARY_COMMANDS: &[&str] = &["frac", "tfrac", "binom", "dbinom", "dfrac", "tbinom", "overset"];

static EMPTY_NODE: LazyLock<TexNode> = LazyLock::new(|| TexNode::new(TexNodeType::Empty, String::new(), None, None));

fn get_command_param_num(command: &str) -> usize {
    if UNARY_COMMANDS.contains(&command) {
        1
    } else if BINARY_COMMANDS.contains(&command) {
        2
    } else {
        0
    }
}

static LEFT_CURLY_BRACKET: LazyLock<TexToken> = LazyLock::new(|| TexToken::new(TexTokenType::Control, "{".to_string()));
static RIGHT_CURLY_BRACKET: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Control, "}".to_string()));

static LEFT_SQUARE_BRACKET: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Element, "[".to_string()));
static RIGHT_SQUARE_BRACKET: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Element, "]".to_string()));

fn eat_whitespaces(tokens: &[TexToken], start: usize) -> &[TexToken] {
    let mut pos = start;
    while pos < tokens.len() && matches!(tokens[pos].token_type, TexTokenType::Space | TexTokenType::Newline) {
        pos += 1;
    }
    &tokens[start..pos]
}

fn eat_parenthesis(tokens: &[TexToken], start: usize) -> Option<&TexToken> {
    let first_token = &tokens[start];
    if first_token.token_type == TexTokenType::Element
        && ["(", ")", "[", "]", "|", "\\{", "\\}", "."].contains(&first_token.value.as_str())
    {
        Some(first_token)
    } else if first_token.token_type == TexTokenType::Command
        && ["lfloor", "rfloor", "lceil", "rceil", "langle", "rangle"].contains(&&first_token.value[1..])
    {
        Some(first_token)
    } else {
        None
    }
}

fn eat_primes(tokens: &[TexToken], start: usize) -> usize {
    let mut pos = start;
    while pos < tokens.len() && tokens[pos] == TexToken::new(TexTokenType::Element, "'".to_string()) {
        pos += 1;
    }
    pos - start
}

fn eat_command_name(latex: &str, start: usize) -> &str {
    let mut pos = start;
    while pos < latex.len() && latex[pos..].chars().next().unwrap().is_alphabetic() {
        pos += 1;
    }
    &latex[start..pos]
}

fn find_closing_match(tokens: &[TexToken], start: usize, left_token: &TexToken, right_token: &TexToken) -> isize {
    assert!(tokens[start].eq(left_token));
    let mut count = 1;
    let mut pos = start + 1;

    while count > 0 {
        if pos >= tokens.len() {
            return -1;
        }
        if tokens[pos].eq(left_token) {
            count += 1;
        } else if tokens[pos].eq(right_token) {
            count -= 1;
        }
        pos += 1;
    }

    (pos - 1) as isize
}

static LEFT_COMMAND: LazyLock<TexToken> = LazyLock::new(|| TexToken::new(TexTokenType::Command, "\\left".to_string()));
static RIGHT_COMMAND: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Command, "\\right".to_string()));

fn find_closing_right_command(tokens: &[TexToken], start: usize) -> isize {
    find_closing_match(tokens, start, &LEFT_COMMAND, &RIGHT_COMMAND)
}

static BEGIN_COMMAND: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Command, "\\begin".to_string()));
static END_COMMAND: LazyLock<TexToken> = LazyLock::new(|| TexToken::new(TexTokenType::Command, "\\end".to_string()));

fn find_closing_end_command(tokens: &[TexToken], start: usize) -> isize {
    find_closing_match(tokens, start, &BEGIN_COMMAND, &END_COMMAND)
}

fn find_closing_curly_bracket_char(latex: &str, start: usize) -> Result<usize, &'static str> {
    assert_eq!(latex[start..].chars().next().unwrap(), '{');
    let mut count = 1;
    let mut pos = start + 1;

    while count > 0 {
        if pos >= latex.len() {
            return Err("Unmatched curly brackets");
        }
        if pos + 1 < latex.len() && ["\\{", "\\}"].contains(&&latex[pos..pos + 2]) {
            pos += 2;
            continue;
        }
        match latex[pos..].chars().next().unwrap() {
            '{' => count += 1,
            '}' => count -= 1,
            _ => {}
        }
        pos += 1;
    }

    Ok(pos - 1)
}

pub fn tokenize(latex: &str) -> Result<Vec<TexToken>, &'static str> {
    let mut tokens: Vec<TexToken> = Vec::new();
    let mut pos = 0;

    while pos < latex.len() {
        let first_char = latex[pos..].chars().next().unwrap();
        let token: TexToken;
        match first_char {
            '%' => {
                let mut new_pos = pos + 1;
                while new_pos < latex.len() && latex[new_pos..].chars().next().unwrap() != '\n' {
                    new_pos += 1;
                }
                token = TexToken::new(TexTokenType::Comment, latex[pos + 1..new_pos].to_string());
                pos = new_pos;
            }
            '{' | '}' | '_' | '^' | '&' => {
                token = TexToken::new(TexTokenType::Control, first_char.to_string());
                pos += 1;
            }
            '\n' => {
                token = TexToken::new(TexTokenType::Newline, first_char.to_string());
                pos += 1;
            }
            '\r' => {
                if pos + 1 < latex.len() && latex[pos + 1..].chars().next().unwrap() == '\n' {
                    token = TexToken::new(TexTokenType::Newline, "\n".to_string());
                    pos += 2;
                } else {
                    token = TexToken::new(TexTokenType::Newline, "\n".to_string());
                    pos += 1;
                }
            }
            ' ' => {
                let mut new_pos = pos;
                while new_pos < latex.len() && latex[new_pos..].chars().next().unwrap() == ' ' {
                    new_pos += 1;
                }
                token = TexToken::new(TexTokenType::Space, latex[pos..new_pos].to_string());
                pos = new_pos;
            }
            '\\' => {
                if pos + 1 >= latex.len() {
                    return Err("Expecting command name after \\");
                }
                let first_two_chars = &latex[pos..pos + 2];
                if ["\\\\", "\\,"].contains(&first_two_chars) {
                    token = TexToken::new(TexTokenType::Control, first_two_chars.to_string());
                } else if ["\\{", "\\}", "\\%", "\\$", "\\&", "\\#", "\\_", "\\|"].contains(&first_two_chars) {
                    token = TexToken::new(TexTokenType::Element, first_two_chars.to_string());
                } else {
                    let command = eat_command_name(latex, pos + 1);
                    token = TexToken::new(TexTokenType::Command, format!("\\{}", command));
                }
                pos += token.value.len();
            }
            _ => {
                if first_char.is_digit(10) {
                    let mut new_pos = pos;
                    while new_pos < latex.len() && latex[new_pos..].chars().next().unwrap().is_digit(10) {
                        new_pos += 1;
                    }
                    token = TexToken::new(TexTokenType::Element, latex[pos..new_pos].to_string());
                } else if first_char.is_alphabetic() {
                    token = TexToken::new(TexTokenType::Element, first_char.to_string());
                } else if "+-*/='<>!.,;:?()[]|".contains(first_char) {
                    token = TexToken::new(TexTokenType::Element, first_char.to_string());
                } else if "~".contains(first_char) {
                    token = TexToken::new(TexTokenType::NoBreakSpace, "space.nobreak".to_string());
                } else {
                    token = TexToken::new(TexTokenType::Unknown, first_char.to_string());
                }
                pos += token.value.len();
            }
        }

        tokens.push(token.clone());

        if token.token_type == TexTokenType::Command
            && ["\\text", "\\operatorname", "\\begin", "\\end"].contains(&token.value.as_str())
        {
            if pos >= latex.len() || latex[pos..].chars().next().unwrap() != '{' {
                // return Err(format!("No content for {} command", token.value));
                panic!("No content for {} command", token.value);
            }
            tokens.push(TexToken::new(TexTokenType::Control, "{".to_string()));
            let pos_closing_bracket = find_closing_curly_bracket_char(latex, pos)?;
            pos += 1;
            let mut text_inside = latex[pos..pos_closing_bracket].to_string();
            let chars = ['{', '}', '\\', '$', '&', '#', '_', '%'];
            for &char in &chars {
                text_inside = text_inside.replace(&format!("\\{}", char), &char.to_string());
            }
            tokens.push(TexToken::new(TexTokenType::Text, text_inside));
            tokens.push(TexToken::new(TexTokenType::Control, "}".to_string()));
            pos = pos_closing_bracket + 1;
        }
    }
    Ok(tokens)
}

type ParseResult = Result<(TexNode, usize), &'static str>;

static SUB_SYMBOL: LazyLock<TexToken> = LazyLock::new(|| TexToken::new(TexTokenType::Control, "_".to_string()));
static SUP_SYMBOL: LazyLock<TexToken> = LazyLock::new(|| TexToken::new(TexTokenType::Control, "^".to_string()));

pub struct LatexParser {
    space_sensitive: bool,
    newline_sensitive: bool,
}

impl LatexParser {
    pub fn new(space_sensitive: bool, newline_sensitive: bool) -> Self {
        LatexParser {
            space_sensitive,
            newline_sensitive,
        }
    }

    pub fn parse(&self, tokens: Vec<TexToken>) -> Result<TexNode, &'static str> {
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
                panic!("Unexpected & outside of an alignment");
            }
            results.push(res);
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
        let first_token = &tokens[start];
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
                            panic!("Unmatched '{{'");
                        }
                        let expr_inside = &tokens[start + 1..pos_closing_bracket as usize];
                        Ok((self.parse(expr_inside.to_vec())?, pos_closing_bracket as usize + 1))
                    }
                    "}" => panic!("Unmatched '}}'"),
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
                    _ => Err("Unknown control sequence"),
                }
            }
            TexTokenType::Unknown => Ok((
                TexNode::new(TexNodeType::Unknown, first_token.value.clone(), None, None),
                start + 1,
            )),
        }
    }

    fn parse_command_expr(&self, tokens: &[TexToken], start: usize) -> ParseResult {
        let command = &tokens[start].value; // command name starts with a \\
        let pos = start + 1;

        if ["left", "right", "begin", "end"].contains(&&command[1..]) {
            panic!("Unexpected command: {}", command);
        }

        let param_num = get_command_param_num(&command[1..]);
        match param_num {
            0 => {
                if !get_symbol_map().contains_key(&command[1..]) {
                    return Ok((
                        TexNode::new(TexNodeType::UnknownMacro, command.clone(), None, None),
                        pos,
                    ));
                }
                Ok((TexNode::new(TexNodeType::Symbol, command.clone(), None, None), pos))
            }
            1 => {
                if pos >= tokens.len() {
                    panic!("Expecting argument for {}", command);
                }
                if command == "\\sqrt" && pos < tokens.len() && tokens[pos] == *LEFT_SQUARE_BRACKET {
                    let pos_left_square_bracket = pos;
                    let pos_right_square_bracket =
                        find_closing_match(tokens, pos, &LEFT_SQUARE_BRACKET, &RIGHT_SQUARE_BRACKET);
                    if pos_right_square_bracket == -1 {
                        panic!("No matching right square bracket for [");
                    }
                    let expr_inside = &tokens[pos_left_square_bracket + 1..pos_right_square_bracket as usize];
                    let exponent = self.parse(expr_inside.to_vec())?;
                    let (arg1, new_pos) =
                        self.parse_next_expr_without_supsub(tokens, pos_right_square_bracket as usize + 1)?;
                    return Ok((
                        TexNode::new(
                            TexNodeType::UnaryFunc,
                            command.clone(),
                            Some(vec![arg1]),
                            Some(Box::from(Sqrt(exponent))),
                        ),
                        new_pos,
                    ));
                } else if command == "\\text" {
                    if pos + 2 >= tokens.len() {
                        panic!("Expecting content for \\text command");
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
            2 => {
                let (arg1, pos1) = self.parse_next_expr_without_supsub(tokens, pos)?;
                let (arg2, pos2) = self.parse_next_expr_without_supsub(tokens, pos1)?;
                Ok((
                    TexNode::new(TexNodeType::BinaryFunc, command.clone(), Some(vec![arg1, arg2]), None),
                    pos2,
                ))
            }
            _ => Err("Invalid number of parameters"),
        }
    }

    fn parse_left_right_expr(&self, tokens: &[TexToken], start: usize) -> ParseResult {
        assert!(tokens[start].eq(&LEFT_COMMAND));

        let mut pos = start + 1;
        pos += eat_whitespaces(tokens, pos).len();

        if pos >= tokens.len() {
            return Err("Expecting delimiter after \\left");
        }

        let left_delimiter = eat_parenthesis(tokens, pos);
        if left_delimiter.is_none() {
            return Err("Invalid delimiter after \\left");
        }
        pos += 1;
        let expr_inside_start = pos;
        let idx = find_closing_right_command(tokens, start);
        if idx == -1 {
            return Err("No matching \\right");
        }
        let expr_inside_end = idx as usize;
        pos = expr_inside_end + 1;

        pos += eat_whitespaces(tokens, pos).len();
        if pos >= tokens.len() {
            return Err("Expecting \\right after \\left");
        }

        let right_delimiter = eat_parenthesis(tokens, pos);
        if right_delimiter.is_none() {
            return Err("Invalid delimiter after \\right");
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

        pos += eat_whitespaces(tokens, pos).len(); // ignore whitespaces and '\n' after \begin{envName}

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
            return Err("Mismatched \\begin and \\end environments");
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

    fn parse_aligned(&self, tokens: &[TexToken]) -> Result<Vec<Vec<TexNode>>, &'static str> {
        let mut pos = 0;
        let mut all_rows: Vec<Vec<TexNode>> = Vec::new();
        let mut row: Vec<TexNode> = Vec::new();
        all_rows.push(row.clone());
        let mut group = TexNode::new(TexNodeType::Ordgroup, String::new(), Some(Vec::new()), None);
        row.push(group.clone());

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
                row = Vec::new();
                group = TexNode::new(TexNodeType::Ordgroup, String::new(), Some(Vec::new()), None);
                row.push(group.clone());
                all_rows.push(row.clone());
            } else if res.node_type == TexNodeType::Control && res.content == "&" {
                group = TexNode::new(TexNodeType::Ordgroup, String::new(), Some(Vec::new()), None);
                row.push(group.clone());
            } else {
                group.args.as_mut().unwrap().push(res);
            }
        }
        Ok(all_rows)
    }
}

// Remove all whitespace before or after _ or ^
fn pass_ignore_whitespace_before_script_mark(tokens: Vec<TexToken>) -> Vec<TexToken> {
    let is_script_mark = |token: &TexToken| token.eq(&SUB_SYMBOL) || token.eq(&SUP_SYMBOL);
    let mut out_tokens: Vec<TexToken> = Vec::new();

    for i in 0..tokens.len() {
        if tokens[i].token_type == TexTokenType::Space && i + 1 < tokens.len() && is_script_mark(&tokens[i + 1]) {
            continue;
        }
        if tokens[i].token_type == TexTokenType::Space && i > 0 && is_script_mark(&tokens[i - 1]) {
            continue;
        }
        out_tokens.push(tokens[i].clone());
    }

    out_tokens
}

fn pass_expand_custom_tex_macros(
    tokens: Vec<TexToken>,
    custom_tex_macros: &std::collections::HashMap<String, String>,
) -> Vec<TexToken> {
    let mut out_tokens: Vec<TexToken> = Vec::new();
    for token in tokens {
        if token.token_type == TexTokenType::Command {
            if let Some(expansion) = custom_tex_macros.get(&token.value) {
                if let Ok(expanded_tokens) = tokenize(expansion) {
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

pub fn parse_tex(tex: &str, custom_tex_macros: &std::collections::HashMap<String, String>) -> Result<TexNode, &'static str> {
    let parser = LatexParser::new(false, false);
    let mut tokens = tokenize(tex)?;
    tokens = pass_ignore_whitespace_before_script_mark(tokens);
    tokens = pass_expand_custom_tex_macros(tokens, custom_tex_macros);
    parser.parse(tokens)
}
