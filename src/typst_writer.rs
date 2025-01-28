use crate::definitions::{TypstNode, TypstNodeData, TypstNodeType, TypstToken, TypstTokenType};
use regex::Regex;
use std::sync::LazyLock;

static TYPST_LEFT_PARENTHESIS: LazyLock<TypstToken> = LazyLock::new(|| TypstToken {
    token_type: TypstTokenType::Element,
    value: "(".to_string(),
});

static TYPST_RIGHT_PARENTHESIS: LazyLock<TypstToken> = LazyLock::new(|| TypstToken {
    token_type: TypstTokenType::Element,
    value: ")".to_string(),
});

static TYPST_COMMA: LazyLock<TypstToken> = LazyLock::new(|| TypstToken {
    token_type: TypstTokenType::Element,
    value: ",".to_string(),
});

static TYPST_NEWLINE: LazyLock<TypstToken> = LazyLock::new(|| TypstToken {
    token_type: TypstTokenType::Symbol,
    value: "\n".to_string(),
});

pub struct TypstWriter {
    non_strict: bool,
    keep_spaces: bool,
    buffer: String,
    queue: Vec<TypstToken>,
    inside_function_depth: usize,
}

impl TypstWriter {
    pub fn new(non_strict: bool, keep_spaces: bool) -> Self {
        TypstWriter {
            non_strict,
            keep_spaces,
            buffer: String::new(),
            queue: Vec::new(),
            inside_function_depth: 0,
        }
    }

    fn write_buffer(&mut self, token: &TypstToken) {
        let str = token.to_string();

        if str.is_empty() {
            return;
        }

        let mut no_need_space = false;
        no_need_space |= self.buffer.ends_with(&['(', '[', '|']) && str.starts_with(char::is_alphanumeric);
        no_need_space |= str.starts_with(&[')', '}', ']', '|']);
        no_need_space |= str.starts_with(&['(', '_', '^', ',', ';', '!']);
        no_need_space |= str == "'";
        no_need_space |= self.buffer.ends_with(char::is_numeric) && str.starts_with(char::is_numeric);
        no_need_space |= self.buffer.ends_with(&['(', '[', '{']) && str.starts_with(&['-', '+'])
            || self.buffer == "-"
            || self.buffer == "+";
        no_need_space |= str.starts_with('\n');
        no_need_space |= self.buffer.is_empty();
        no_need_space |= str.starts_with(char::is_whitespace);
        no_need_space |= self.buffer.ends_with('&') && str == "=";
        no_need_space |= self.buffer.ends_with(&[' ', '_', '^', '{', '(']);

        if !no_need_space {
            self.buffer.push(' ');
        }

        self.buffer.push_str(&str);
    }

    // Serialize a tree of TypstNode into a list of TypstToken
    pub fn serialize(&mut self, node: &TypstNode) {
        match node.node_type {
            TypstNodeType::Empty => {}
            TypstNodeType::Atom => {
                if node.content == "," && self.inside_function_depth > 0 {
                    self.queue
                        .push(TypstToken::new(TypstTokenType::Symbol, "comma".to_string()));
                } else {
                    self.queue
                        .push(TypstToken::new(TypstTokenType::Element, node.content.clone()));
                }
            }
            TypstNodeType::Symbol => {
                self.queue
                    .push(TypstToken::new(TypstTokenType::Symbol, node.content.clone()));
            }
            TypstNodeType::Text => {
                self.queue
                    .push(TypstToken::new(TypstTokenType::Text, node.content.clone()));
            }
            TypstNodeType::Comment => {
                self.queue
                    .push(TypstToken::new(TypstTokenType::Comment, node.content.clone()));
            }
            TypstNodeType::Whitespace => {
                for c in node.content.chars() {
                    if c == ' ' {
                        if self.keep_spaces {
                            self.queue.push(TypstToken::new(TypstTokenType::Space, c.to_string()));
                        }
                    } else if c == '\n' {
                        self.queue.push(TypstToken::new(TypstTokenType::Symbol, c.to_string()));
                    } else {
                        panic!("Unexpected whitespace character: {}", c);
                    }
                }
            }
            TypstNodeType::Group => {
                if let Some(args) = &node.args {
                    for item in args {
                        self.serialize(item);
                    }
                }
            }
            TypstNodeType::Supsub => {
                if let TypstNodeData::Supsub(data) = node.data.as_ref().unwrap().as_ref() {
                    self.append_with_brackets_if_needed(&data.base);

                    let mut trailing_space_needed = false;
                    let has_prime = data
                        .sup
                        .as_ref()
                        .map_or(false, |sup| sup.node_type == TypstNodeType::Atom && sup.content == "'");
                    if has_prime {
                        self.queue
                            .push(TypstToken::new(TypstTokenType::Element, "'".to_string()));
                        trailing_space_needed = false;
                    }
                    if let Some(sub) = &data.sub {
                        self.queue
                            .push(TypstToken::new(TypstTokenType::Element, "_".to_string()));
                        trailing_space_needed = self.append_with_brackets_if_needed(sub);
                    }
                    if let Some(sup) = &data.sup {
                        if !has_prime {
                            self.queue
                                .push(TypstToken::new(TypstTokenType::Element, "^".to_string()));
                            trailing_space_needed = self.append_with_brackets_if_needed(sup);
                        }
                    }
                    if trailing_space_needed {
                        self.queue
                            .push(TypstToken::new(TypstTokenType::Control, " ".to_string()));
                    }
                }
            }
            TypstNodeType::FuncCall => {
                self.queue
                    .push(TypstToken::new(TypstTokenType::Symbol, node.content.clone()));
                self.inside_function_depth += 1;
                self.queue.push(TYPST_LEFT_PARENTHESIS.clone());
                if let Some(args) = &node.args {
                    for (i, arg) in args.iter().enumerate() {
                        self.serialize(arg);
                        if i < args.len() - 1 {
                            self.queue
                                .push(TypstToken::new(TypstTokenType::Element, ",".to_string()));
                        }
                    }
                }
                if let Some(options) = &node.options {
                    for (key, value) in options {
                        self.queue
                            .push(TypstToken::new(TypstTokenType::Symbol, format!(", {}: {}", key, value)));
                    }
                }
                self.queue.push(TYPST_RIGHT_PARENTHESIS.clone());
                self.inside_function_depth -= 1;
            }
            TypstNodeType::Align => {
                if let TypstNodeData::Array(matrix) = node.data.as_ref().unwrap().as_ref() {
                    for (i, row) in matrix.iter().enumerate() {
                        for (j, cell) in row.iter().enumerate() {
                            if j > 0 {
                                self.queue
                                    .push(TypstToken::new(TypstTokenType::Element, "&".to_string()));
                            }
                            self.serialize(cell);
                        }
                        if i < matrix.len() - 1 {
                            self.queue
                                .push(TypstToken::new(TypstTokenType::Symbol, "\\".to_string()));
                        }
                    }
                }
            }
            TypstNodeType::Matrix => {
                if let TypstNodeData::Array(matrix) = node.data.as_ref().unwrap().as_ref() {
                    self.queue
                        .push(TypstToken::new(TypstTokenType::Symbol, "mat".to_string()));
                    self.inside_function_depth += 1;
                    self.queue.push(TYPST_LEFT_PARENTHESIS.clone());
                    if let Some(options) = &node.options {
                        for (key, value) in options {
                            self.queue
                                .push(TypstToken::new(TypstTokenType::Symbol, format!("{}: {}, ", key, value)));
                        }
                    }
                    for (i, row) in matrix.iter().enumerate() {
                        for (j, cell) in row.iter().enumerate() {
                            self.serialize(cell);
                            if j < row.len() - 1 {
                                self.queue
                                    .push(TypstToken::new(TypstTokenType::Element, ",".to_string()));
                            } else if i < matrix.len() - 1 {
                                self.queue
                                    .push(TypstToken::new(TypstTokenType::Element, ";".to_string()));
                            }
                        }
                    }
                    self.queue.push(TYPST_RIGHT_PARENTHESIS.clone());
                    self.inside_function_depth -= 1;
                }
            }
            TypstNodeType::Unknown => {
                if self.non_strict {
                    self.queue
                        .push(TypstToken::new(TypstTokenType::Symbol, node.content.clone()));
                } else {
                    panic!("Unknown macro: {}", node.content);
                }
            }
            _ => panic!("Unimplemented node type to append: {:?}", node.node_type),
        }
    }

    fn append_with_brackets_if_needed(&mut self, node: &TypstNode) -> bool {
        let mut need_to_wrap = matches!(
            node.node_type,
            TypstNodeType::Group | TypstNodeType::Supsub | TypstNodeType::Empty
        );

        if node.node_type == TypstNodeType::Group {
            if let Some(args) = &node.args {
                let first = &args[0];
                let last = &args[args.len() - 1];
                if is_delimiter(first) && is_delimiter(last) {
                    need_to_wrap = false;
                }
            }
        }

        if need_to_wrap {
            self.queue.push(TYPST_LEFT_PARENTHESIS.clone());
            self.serialize(node);
            self.queue.push(TYPST_RIGHT_PARENTHESIS.clone());
        } else {
            self.serialize(node);
        }

        !need_to_wrap
    }

    fn flush_queue(&mut self) {
        let soft_space = TypstToken::new(TypstTokenType::Control, " ".to_string());

        // delete soft spaces if they are not needed
        let queue_len = self.queue.len();
        for i in 0..queue_len {
            let is_soft_space = self.queue[i].eq(&soft_space);
            if is_soft_space {
                if i == queue_len - 1 {
                    self.queue[i].value = "".to_string();
                } else {
                    let next_is_end = self.queue[i + 1] == *TYPST_LEFT_PARENTHESIS
                        || self.queue[i + 1] == *TYPST_COMMA
                        || self.queue[i + 1] == *TYPST_NEWLINE;
                    if next_is_end {
                        self.queue[i].value = "".to_string();
                    }
                }
            }
        }

        // for token in &mut self.queue {
        //     self.write_buffer(token);
        // }
        let queue = std::mem::take(&mut self.queue);
        for token in queue {
            self.write_buffer(&token);
        }

        self.queue.clear();
    }

    pub fn finalize(&mut self) -> String {
        self.flush_queue();

        let smart_floor_pass = |input: &str| -> String {
            let re = Regex::new(r"floor\.l\s*(.*?)\s*floor\.r").unwrap();
            let mut res = re.replace_all(input, "floor($1)").to_string();
            res = res.replace("floor()", "floor(\"\")");
            res
        };

        let smart_ceil_pass = |input: &str| -> String {
            let re = Regex::new(r"ceil\.l\s*(.*?)\s*ceil\.r").unwrap();
            let mut res = re.replace_all(input, "ceil($1)").to_string();
            res = res.replace("ceil()", "ceil(\"\")");
            res
        };

        let smart_round_pass = |input: &str| -> String {
            let re = Regex::new(r"floor\.l\s*(.*?)\s*ceil\.r").unwrap();
            let mut res = re.replace_all(input, "round($1)").to_string();
            res = res.replace("round()", "round(\"\")");
            res
        };

        let all_passes = [smart_floor_pass, smart_ceil_pass, smart_round_pass];
        for pass in &all_passes {
            self.buffer = pass(&self.buffer);
        }

        self.buffer.clone()
    }
}

fn is_delimiter(c: &TypstNode) -> bool {
    matches!(c.node_type, TypstNodeType::Atom)
        && ["(", ")", "[", "]", "{", "}", "|", "⌊", "⌋", "⌈", "⌉"].contains(&c.content.as_str())
}
