use crate::definitions::{TexNode, TexToken, TexTokenType};
use std::collections::HashMap;

pub const UNARY_COMMANDS: &[&'static str] = &[
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
    "floor", // This is a custom macro
];

pub const OPTION_BINARY_COMMANDS: &[&'static str] = &["sqrt"];

pub const BINARY_COMMANDS: &[&'static str] = &["frac", "tfrac", "binom", "dbinom", "dfrac", "tbinom", "overset"];

pub type ExpandResult = Result<(Vec<TexToken>, usize), String>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CommandType {
    Symbol,
    Unary,
    Binary,
    OptionalBinary,
}

pub struct CustomMacro {
    pub name: String,
    pub command_type: CommandType,
    pub implementation: Box<dyn Fn(&Vec<Vec<TexToken>>) -> Vec<TexToken>>,
}

#[derive(Default)]
pub struct CommandRegistry {
    custom_macros: Vec<CustomMacro>,
    custom_macro_names: HashMap<String, CommandType>,
}

impl CommandRegistry {
    pub fn new() -> CommandRegistry {
        Self::default()
    }

    pub fn register_custom_macro(
        &mut self,
        name: &str,
        command_type: CommandType,
        implementation: Box<dyn Fn(&Vec<Vec<TexToken>>) -> Vec<TexToken>>,
    ) {
        self.custom_macros.push(CustomMacro {
            name: name.to_string(),
            command_type,
            implementation,
        });
        self.custom_macro_names.insert(name.to_string(), command_type);
    }

    pub fn get_command_type(&self, command_name: &str) -> Option<CommandType> {
        if UNARY_COMMANDS.contains(&command_name) {
            Some(CommandType::Unary)
        } else if BINARY_COMMANDS.contains(&command_name) {
            Some(CommandType::Binary)
        } else if OPTION_BINARY_COMMANDS.contains(&command_name) {
            Some(CommandType::OptionalBinary)
        } else if self.custom_macro_names.contains_key(command_name) {
            self.custom_macro_names.get(command_name).copied()
        } else {
            // fallback to symbol (no arguments)
            Some(CommandType::Symbol)
        }
    }

    pub fn expand_macros(&self, tokens: &[TexToken]) -> Result<Vec<TexToken>, String> {
        let mut expanded_tokens: Vec<TexToken> = Vec::new();
        let mut pos: usize = 0;

        while pos < tokens.len() {
            let token = &tokens[pos];
            if token.token_type == TexTokenType::Command {
                if let Some(custom_macro) = self.custom_macros.iter().find(|macro_| macro_.name == token.value[1..]) {
                    let (expanded_command, new_pos) = self.expand_command(tokens, custom_macro, pos)?;
                    expanded_tokens.extend(expanded_command);
                    pos = new_pos;
                } else {
                    expanded_tokens.push(token.clone());
                    pos += 1;
                }
            } else {
                expanded_tokens.push(token.clone());
                pos += 1;
            }
        }
        Ok(expanded_tokens)
    }

    // this will get called recursively
    fn expand_command(&self, tokens: &[TexToken], custom_macro: &CustomMacro, start: usize) -> ExpandResult {
        let command_name = &tokens[start].value[1..]; // remove the backslash
        assert_eq!(command_name, custom_macro.name);
        let command_type = custom_macro.command_type;
        let mut pos = start + 1; // come to what comes after the command
        let mut arguments: Vec<Vec<TexToken>> = Vec::new();

        match command_type {
            CommandType::Symbol => {
                // no arguments, don't move the pos
            }
            CommandType::Unary => {
                if !tokens[pos].value.eq("{") {
                    return Err(format!("Expecting one argument for command {}", command_name));
                }
                pos += 1;
                if let Some(right_curly_bracket) = tokens[pos..].iter().position(|token| token.value == "}") {
                    let new_pos = pos + right_curly_bracket;
                    let argument: &[TexToken] = &tokens[pos..new_pos];
                    arguments.push(self.expand_macros(argument)?);
                    pos = new_pos + 1;
                } else {
                    return Err(format!("Unmatched curly brackets for command {}", command_name));
                }
            }
            CommandType::Binary => {
                if !tokens[pos].value.eq("{") {
                    return Err(format!("No argument provided for command {}", command_name));
                }
                pos += 1;
                if let Some(right_curly_bracket) = tokens[pos..].iter().position(|token| token.value == "}") {
                    let new_pos = pos + right_curly_bracket;
                    let first_argument: &[TexToken] = &tokens[pos..new_pos];
                    arguments.push(self.expand_macros(first_argument)?);
                    pos = new_pos;
                } else {
                    return Err(format!("Unmatched curly brackets for command {}", command_name));
                }
                pos += 1;

                if !tokens[pos].value.eq("{") {
                    return Err(format!("Expecting two arguments for command {}", command_name));
                }
                pos += 1;
                if let Some(right_curly_bracket) = tokens[pos..].iter().position(|token| token.value == "}") {
                    let new_pos = pos + right_curly_bracket;
                    let second_argument: &[TexToken] = &tokens[pos..new_pos];
                    arguments.push(self.expand_macros(second_argument)?);
                    pos = new_pos;
                } else {
                    return Err(format!("Unmatched curly brackets for command {}", command_name));
                }
                pos += 1;
            }
            CommandType::OptionalBinary => {
                match tokens[pos].value.as_str() {
                    "[" => {
                        // one optional argument, one mandatory argument
                        pos += 1;
                        if let Some(right_square_bracket) = tokens[pos..].iter().position(|token| token.value == "]") {
                            let new_pos = pos + right_square_bracket;
                            let optional_argument: &[TexToken] = &tokens[pos..new_pos];
                            arguments.push(self.expand_macros(optional_argument)?);
                            pos = new_pos;
                        } else {
                            return Err(format!("Unmatched square brackets for command {}", command_name));
                        }
                        pos += 1;

                        if !tokens[pos].value.eq("{") {
                            return Err(format!("Expecting the mandatory argument for command {}", command_name));
                        }
                        pos += 1;
                        if let Some(right_curly_bracket) = tokens[pos..].iter().position(|token| token.value == "}") {
                            let new_pos = pos + right_curly_bracket;
                            let mandatory_argument: &[TexToken] = &tokens[pos..new_pos];
                            arguments.push(self.expand_macros(mandatory_argument)?);
                            pos = new_pos;
                        } else {
                            return Err(format!("Unmatched curly brackets for command {}", command_name));
                        }
                    }
                    "{" => {
                        // no optional argument, one mandatory argument
                        pos += 1;
                        if let Some(right_curly_bracket) = tokens[pos..].iter().position(|token| token.value == "}") {
                            let new_pos = pos + right_curly_bracket;
                            let mandatory_argument: &[TexToken] = &tokens[pos..new_pos];
                            arguments.push(self.expand_macros(mandatory_argument)?);
                            pos = new_pos;
                        } else {
                            return Err(format!("Unmatched curly brackets for command {}", command_name));
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Expecting optional or mandatory argument for command {}",
                            command_name
                        ));
                    }
                };
            }
        }

        let expanded_tokens = (custom_macro.implementation)(&arguments);
        Ok((expanded_tokens, pos))
    }
}

pub fn parse_custom_macros(latex: &str) -> Result<Vec<CustomMacro>, String> {
    let latex: Vec<char> = latex.chars().collect();
    let pattern: Vec<char> = "\\newcommand".chars().collect();
    let mut pos = 0;
    let mut custom_macros: Vec<CustomMacro> = Vec::new();

    while pos < latex.len().saturating_sub(pattern.len()) {
        if latex[pos..pos + pattern.len()] == pattern[..] {
            // extract the new command name
            let new_command_name: String;
            if !latex[pos] == '{' {
                return Err("Expecting { after \\newcommand".to_string());
            }
            pos += 1;
            if !latex[pos] == "\\" {
                return Err("Expecting backslash after {".to_string());
            }
            if let Some(right_curly_bracket) = latex[pos..].iter().position(|&c| c == '}') {
                new_command_name = latex[pos..pos + right_curly_bracket].iter().collect();
                pos += right_curly_bracket;
            } else {
                return Err("Unmatched curly brackets".to_string());
            }

            // check if there is a specification of number of arguments
            let num_of_args: usize;
            pos += 1;
            if latex[pos] == '[' {
                pos += 1;
                if let Some(right_square_bracket) = latex[pos..].iter().position(|&c| c == ']') {
                    num_of_args = latex[pos..pos + right_square_bracket]
                        .iter()
                        .collect::<String>()
                        .parse::<usize>()
                        .map_err(|e| e.to_string())?;
                    if num_of_args > 2 {
                        return Err("Only unary and binary commands are supported".to_string());
                    }
                    pos += right_square_bracket;
                } else {
                    return Err("Unmatched square brackets".to_string());
                }
            } else {
                num_of_args = 0;
            }

            // check if there is a default value for the first argument
            let default_value: Option<String>;
            pos += 1;
            if latex[pos] == '[' {
                pos += 1;
                if let Some(right_square_bracket) = latex[pos..].iter().position(|&c| c == ']') {
                    default_value = latex[pos..pos + right_square_bracket]
                        .iter()
                        .collect::<Option<String>>();
                    pos += right_square_bracket;
                } else {
                    return Err("Unmatched square brackets".to_string());
                }
            } else {
                default_value = None;
            }

            // extract the definition
            let definition: String;
            pos += 1;
            if !latex[pos] == '{' {
                return Err("Expecting { before the definition".to_string());
            }
            pos += 1;
            if let Some(right_curly_bracket) = latex[pos..].iter().position(|&c| c == '}') {
                definition = latex[pos..pos + right_curly_bracket].iter().collect();
                pos += right_curly_bracket;
            } else {
                return Err("Unmatched curly brackets".to_string());
            }

            custom_macros.push(construct_custom_macro(
                new_command_name,
                num_of_args,
                default_value,
                definition,
            )?);
        }
        pos += 1;
    }

    todo!("idealy, it should accept raw latex \newcommand definitions, but may be hard to parse");
}

fn construct_custom_macro(
    new_command_name: String,
    num_of_args: usize,
    default_value: Option<String>,
    definition: String,
) -> Result<CustomMacro, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::TexTokenType;
    use crate::tex_tokenizer::tokenize;

    #[test]
    fn test_tokenize() {
        let tex = r"\alpha";
        let tokens = tokenize(tex).unwrap();
        assert_eq!(
            tokens,
            vec![TexToken {
                token_type: TexTokenType::Command,
                value: r"\alpha".to_string(),
            }]
        );
    }

    #[test]
    fn test_command_registry_symbol() {
        let mut registry = CommandRegistry::new();

        let implementation = |tokens: &Vec<Vec<TexToken>>| {
            vec![TexToken {
                token_type: TexTokenType::Command,
                value: r"\mycommandexpanded".to_string(),
            }]
        };
        registry.register_custom_macro("mycommand", CommandType::Symbol, Box::new(implementation));

        assert_eq!(registry.get_command_type("mycommand"), Some(CommandType::Symbol));

        let tokens = vec![TexToken {
            token_type: TexTokenType::Command,
            value: r"\mycommand".to_string(),
        }];
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(
            expanded_tokens,
            vec![TexToken {
                token_type: TexTokenType::Command,
                value: r"\mycommandexpanded".to_string(),
            }]
        );
    }

    #[test]
    fn test_command_registry_simple_unary() {
        let mut registry = CommandRegistry::new();

        let implementation = |tokens: &Vec<Vec<TexToken>>| {
            let mut res = tokenize(r"\expanded{").unwrap();
            res.extend(tokens[0].iter().cloned());
            res.push(TexToken {
                token_type: TexTokenType::Control,
                value: "}".to_string(),
            });
            res
        };
        registry.register_custom_macro("mycommand", CommandType::Unary, Box::new(implementation));

        assert_eq!(registry.get_command_type("mycommand"), Some(CommandType::Unary));

        let tokens = tokenize(r"\mycommand{a}").unwrap();
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(expanded_tokens, tokenize(r"\expanded{a}").unwrap(),);
    }
}
