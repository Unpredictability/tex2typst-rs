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
    buffer: String,
    queue: Vec<TypstToken>,
    inside_function_depth: usize,
}

impl TypstWriter {
    pub fn new() -> Self {
        TypstWriter {
            buffer: String::new(),
            queue: Vec::new(),
            inside_function_depth: 0,
        }
    }

    fn write_buffer(&mut self, token: &TypstToken) {
        let new_str = token.to_string();

        if new_str.is_empty() {
            return;
        }

        let mut no_need_space = false;
        // putting the first token in clause
        no_need_space |= self.buffer.ends_with(&['(', '[', '|']) && new_str.starts_with(char::is_alphanumeric);
        // closing a clause
        no_need_space |= new_str.starts_with(&[')', '}', ']', '|']);
        // putting the opening '(' for a function
        no_need_space |= !self.buffer.ends_with('=') && new_str.starts_with('(');
        // putting punctuation
        no_need_space |= new_str.starts_with(&['(', '_', '^', ',', ';', '!']);
        // putting a prime
        no_need_space |= new_str == "'";
        // continue a number
        no_need_space |= self.buffer.ends_with(char::is_numeric) && new_str.starts_with(char::is_numeric);
        // leading sign. e.g. produce "+1" instead of " +1"
        no_need_space |= self.buffer.ends_with(&['(', '[', '{']) && new_str.starts_with(&['-', '+'])
            || self.buffer == "-"
            || self.buffer == "+";
        // new line
        no_need_space |= new_str.starts_with('\n');
        // buffer is empty
        no_need_space |= self.buffer.is_empty();
        // str is starting with a space itself
        no_need_space |= new_str.starts_with(char::is_whitespace);
        // "&=" instead of "& ="
        no_need_space |= self.buffer.ends_with('&') && new_str == "=";
        // before or after a slash e.g. "a/b" instead of "a / b"
        no_need_space |= self.buffer.ends_with('/') || new_str.starts_with('/');
        // other cases
        no_need_space |= self.buffer.ends_with(&[' ', '_', '^', '{', '(']);

        if !no_need_space {
            self.buffer.push(' ');
        }

        self.buffer.push_str(&new_str);
    }

    // Serialize a tree of TypstNode into a list of TypstToken
    pub fn serialize(&mut self, node: &TypstNode) -> Result<(), String> {
        use TypstNodeType as N;
        use TypstTokenType as T;
        match node.node_type {
            N::Empty => {Ok(())}
            N::Atom => {
                if node.content == "," && self.inside_function_depth > 0 {
                    self.queue.push(TypstToken::new(T::Symbol, "comma".to_string()));
                } else {
                    self.queue.push(TypstToken::new(T::Element, node.content.clone()));
                }
                Ok(())
            }
            N::Symbol => {
                self.queue.push(TypstToken::new(T::Symbol, node.content.clone()));
                Ok(())
            }
            N::Text => {
                self.queue.push(TypstToken::new(T::Text, node.content.clone()));
                Ok(())
            }
            N::Comment => {
                self.queue.push(TypstToken::new(T::Comment, node.content.clone()));
                Ok(())
            }
            N::Whitespace => {
                for c in node.content.chars() {
                    if c == ' ' {
                    } else if c == '\n' {
                        self.queue.push(TypstToken::new(T::Symbol, c.to_string()));
                    } else {
                        return Err(format!("Unexpected whitespace character: {}", c));
                    }
                }
                Ok(())
            }
            N::NoBreakSpace => {
                self.queue.push(TypstToken::new(T::Symbol, "space.nobreak".to_string()));
                Ok(())
            }
            N::Group => {
                if let Some(args) = &node.args {
                    for item in args {
                        self.serialize(item)?;
                    }
                }
                Ok(())
            }
            N::Supsub => {
                if let TypstNodeData::Supsub(data) = node.data.as_ref().unwrap().as_ref() {
                    self.append_with_brackets_if_needed(&data.base)?;

                    let mut trailing_space_needed = false;
                    let has_prime = data
                        .sup
                        .as_ref()
                        .map_or(false, |sup| sup.node_type == N::Atom && sup.content == "'");
                    if has_prime {
                        self.queue.push(TypstToken::new(T::Element, "'".to_string()));
                        trailing_space_needed = false;
                    }
                    if let Some(sub) = &data.sub {
                        self.queue.push(TypstToken::new(T::Element, "_".to_string()));
                        trailing_space_needed = self.append_with_brackets_if_needed(sub)?;
                    }
                    if let Some(sup) = &data.sup {
                        if !has_prime {
                            self.queue.push(TypstToken::new(T::Element, "^".to_string()));
                            trailing_space_needed = self.append_with_brackets_if_needed(sup)?;
                        }
                    }
                    if trailing_space_needed {
                        self.queue.push(TypstToken::new(T::Control, " ".to_string()));
                    }
                }
                Ok(())
            }
            N::FuncCall => {
                self.queue.push(TypstToken::new(T::Symbol, node.content.clone()));
                self.inside_function_depth += 1;
                self.queue.push(TYPST_LEFT_PARENTHESIS.clone());
                if let Some(args) = &node.args {
                    for (i, arg) in args.iter().enumerate() {
                        self.serialize(arg)?;
                        if i < args.len() - 1 {
                            self.queue.push(TypstToken::new(T::Element, ",".to_string()));
                        }
                    }
                }
                if let Some(options) = &node.options {
                    for (key, value) in options {
                        self.queue
                            .push(TypstToken::new(T::Symbol, format!(", {}: {}", key, value)));
                    }
                }
                self.queue.push(TYPST_RIGHT_PARENTHESIS.clone());
                self.inside_function_depth -= 1;
                Ok(())
            }
            N::Fraction => {
                let num = &node.args.as_ref().unwrap()[0];
                let den = &node.args.as_ref().unwrap()[1];
                self.smart_parenthesis(num)?;
                self.queue.push(TypstToken::new(T::Symbol, "/".to_string()));
                self.smart_parenthesis(den)?;
                Ok(())
            }
            N::Align => {
                if let TypstNodeData::Array(matrix) = node.data.as_ref().unwrap().as_ref() {
                    for (i, row) in matrix.iter().enumerate() {
                        for (j, cell) in row.iter().enumerate() {
                            if j > 0 {
                                self.queue.push(TypstToken::new(T::Element, "&".to_string()));
                            }
                            self.serialize(cell)?;
                        }
                        if i < matrix.len() - 1 {
                            self.queue.push(TypstToken::new(T::Symbol, "\\".to_string()));
                        }
                    }
                }
                Ok(())
            }
            N::Matrix => {
                if let TypstNodeData::Array(matrix) = node.data.as_ref().unwrap().as_ref() {
                    self.queue.push(TypstToken::new(T::Symbol, "mat".to_string()));
                    self.inside_function_depth += 1;
                    self.queue.push(TYPST_LEFT_PARENTHESIS.clone());
                    if let Some(options) = &node.options {
                        for (key, value) in options {
                            self.queue
                                .push(TypstToken::new(T::Symbol, format!("{}: {}, ", key, value)));
                        }
                    }
                    for (i, row) in matrix.iter().enumerate() {
                        for (j, cell) in row.iter().enumerate() {
                            self.serialize(cell)?;
                            if j < row.len() - 1 {
                                self.queue.push(TypstToken::new(T::Element, ",".to_string()));
                            } else if i < matrix.len() - 1 {
                                self.queue.push(TypstToken::new(T::Element, ";".to_string()));
                            }
                        }
                    }
                    self.queue.push(TYPST_RIGHT_PARENTHESIS.clone());
                    self.inside_function_depth -= 1;
                }
                Ok(())
            }
            N::Unknown => {
                self.queue.push(TypstToken::new(T::Symbol, node.content.clone()));
                Ok(())
            }
        }
    }

    fn smart_parenthesis(&mut self, node: &TypstNode) -> Result<(), String> {
        if node.node_type == TypstNodeType::Group {
            self.queue.push(TYPST_LEFT_PARENTHESIS.clone());
            self.serialize(node)?;
            self.queue.push(TYPST_RIGHT_PARENTHESIS.clone());
        } else {
            self.serialize(node)?;
        }
        Ok(())
    }

    fn append_with_brackets_if_needed(&mut self, node: &TypstNode) -> Result<bool, String> {
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
            self.serialize(node)?;
            self.queue.push(TYPST_RIGHT_PARENTHESIS.clone());
        } else {
            self.serialize(node)?;
        }

        Ok(!need_to_wrap)
    }

    fn flush_queue(&mut self) {
        let soft_space = TypstToken::new(TypstTokenType::Control, " ".to_string());

        // delete soft spaces if they are not needed
        let queue_len = self.queue.len();
        for i in 0..queue_len {
            if self.queue[i].eq(&soft_space) {
                if i == queue_len - 1 {
                    self.queue[i].value = "".to_string();
                } else {
                    let next_is_end = self.queue[i + 1] == *TYPST_RIGHT_PARENTHESIS
                        || self.queue[i + 1] == *TYPST_COMMA
                        || self.queue[i + 1] == *TYPST_NEWLINE;
                    if next_is_end {
                        self.queue[i].value = "".to_string();
                    }
                }
            }
        }

        let queue = std::mem::take(&mut self.queue);
        for token in queue {
            self.write_buffer(&token);
        }

        self.queue.clear();
    }

    pub fn finalize(&mut self) -> Result<String, String> {
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

        Ok(self.buffer.clone())
    }
}

fn is_delimiter(c: &TypstNode) -> bool {
    matches!(c.node_type, TypstNodeType::Atom)
        && ["(", ")", "[", "]", "{", "}", "|", "⌊", "⌋", "⌈", "⌉"].contains(&c.content.as_str())
}
