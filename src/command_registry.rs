use crate::definitions::{TexNode, TexToken, TexTokenType};
use crate::tex_tokenizer::tokenize;
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

pub const BINARY_COMMANDS: &[&'static str] = &["frac", "tfrac", "binom", "dbinom", "dfrac", "tbinom", "overset"];

pub const OPTION_UNARY_COMMANDS: &[&'static str] = &[];

pub const OPTION_BINARY_COMMANDS: &[&'static str] = &["sqrt"];

pub type ExpandResult = Result<(Vec<TexToken>, usize), String>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CommandType {
    Symbol,
    Unary,
    Binary,
    OptionalUnary,
    OptionalBinary,
}

pub struct CustomMacro {
    pub name: String,
    pub command_type: CommandType,
    pub implementation: Box<dyn Fn(&Vec<Vec<TexToken>>) -> Result<Vec<TexToken>, String>>,
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
        implementation: Box<dyn Fn(&Vec<Vec<TexToken>>) -> Result<Vec<TexToken>, String>>,
    ) {
        self.custom_macros.push(CustomMacro {
            name: name.to_string(),
            command_type,
            implementation,
        });
        self.custom_macro_names.insert(name.to_string(), command_type);
    }

    pub fn register_custom_macros(&mut self, custom_macros: Vec<CustomMacro>) {
        for custom_macro in custom_macros {
            self.custom_macro_names
                .insert(custom_macro.name.clone(), custom_macro.command_type);
            self.custom_macros.push(custom_macro);
        }
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
                if let Some(custom_macro) = self.custom_macros.iter().find(|macro_| macro_.name == token.value) {
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
        let command_name = &tokens[start].value; // starts with \
        assert_eq!(command_name, &custom_macro.name);
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
                if let Some(right_curly_bracket_pos) = find_matching_right_curly_bracket_token(tokens, pos) {
                    let argument: &[TexToken] = &tokens[pos..right_curly_bracket_pos];
                    arguments.push(self.expand_macros(argument)?);
                    pos = right_curly_bracket_pos + 1;
                } else {
                    return Err(format!("Unmatched curly brackets for command {}", command_name));
                }
            }
            CommandType::Binary => {
                if !tokens[pos].value.eq("{") {
                    return Err(format!("No argument provided for command {}", command_name));
                }
                pos += 1;
                if let Some(right_curly_bracket_pos) = find_matching_right_curly_bracket_token(tokens, pos) {
                    let first_argument: &[TexToken] = &tokens[pos..right_curly_bracket_pos];
                    arguments.push(self.expand_macros(first_argument)?);
                    pos = right_curly_bracket_pos;
                } else {
                    return Err(format!("Unmatched curly brackets for command {}", command_name));
                }
                pos += 1;

                if !tokens[pos].value.eq("{") {
                    return Err(format!("Expecting two arguments for command {}", command_name));
                }
                pos += 1;
                if let Some(right_curly_bracket_pos) = find_matching_right_curly_bracket_token(tokens, pos) {
                    let second_argument: &[TexToken] = &tokens[pos..right_curly_bracket_pos];
                    arguments.push(self.expand_macros(second_argument)?);
                    pos = right_curly_bracket_pos;
                } else {
                    return Err(format!("Unmatched curly brackets for command {}", command_name));
                }
                pos += 1;
            }
            CommandType::OptionalUnary => {
                match tokens[pos].value.as_str() {
                    "[" => {
                        // one optional argument
                        pos += 1;
                        if let Some(right_square_bracket) = tokens[pos..].iter().position(|token| token.value == "]") {
                            let new_pos = pos + right_square_bracket;
                            let optional_argument: &[TexToken] = &tokens[pos..new_pos];
                            arguments.push(self.expand_macros(optional_argument)?);
                            pos = new_pos + 1;
                        } else {
                            return Err(format!("Unmatched right square brackets for command {}", command_name));
                        }
                    }
                    _ => {
                        // no given optional argument, will use the default value
                    }
                };
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
                            pos += 1;
                        } else {
                            return Err(format!("Unmatched square brackets for command {}", command_name));
                        }

                        if !tokens[pos].value.eq("{") {
                            return Err(format!(
                                "Expecting the mandatory argument after the optional argument for command {}",
                                command_name
                            ));
                        }
                        pos += 1;
                        if let Some(right_curly_bracket_pos) = find_matching_right_curly_bracket_token(tokens, pos) {
                            let mandatory_argument: &[TexToken] = &tokens[pos..right_curly_bracket_pos];
                            arguments.push(self.expand_macros(mandatory_argument)?);
                            pos = right_curly_bracket_pos + 1;
                        } else {
                            return Err(format!("Unmatched curly brackets for command {}", command_name));
                        }
                    }
                    "{" => {
                        // no optional argument, one mandatory argument
                        pos += 1;
                        if let Some(right_curly_bracket_pos) = find_matching_right_curly_bracket_token(tokens, pos) {
                            let mandatory_argument: &[TexToken] = &tokens[pos..right_curly_bracket_pos];
                            arguments.push(self.expand_macros(mandatory_argument)?);
                            pos = right_curly_bracket_pos + 1;
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

        let expanded_tokens = (custom_macro.implementation)(&arguments)?;
        Ok((expanded_tokens, pos))
    }
}

fn find_matching_right_curly_bracket_token(tokens: &[TexToken], start: usize) -> Option<usize> {
    let mut count = 1;
    let mut pos = start + 1;

    while count > 0 {
        if pos >= tokens.len() {
            return None;
        }
        if pos + 1 < tokens.len() && tokens[pos].value == "\\" && tokens[pos + 1].value == "}" {
            pos += 2;
            continue;
        }
        match tokens[pos].value.as_str() {
            "{" => count += 1,
            "}" => count -= 1,
            _ => {}
        }
        pos += 1;
    }

    Some(pos - 1)
}

fn find_matching_right_curly_bracket_char(latex: &Vec<char>, start: usize) -> Option<usize> {
    let mut count = 1;
    let mut pos = start + 1;

    while count > 0 {
        if pos >= latex.len() {
            return None;
        }
        if pos + 1 < latex.len() && latex[pos] == '\\' && latex[pos + 1] == '}' {
            pos += 2;
            continue;
        }
        match latex[pos] {
            '{' => count += 1,
            '}' => count -= 1,
            _ => {}
        }
        pos += 1;
    }

    Some(pos - 1)
}

pub fn parse_custom_macros(latex: &str) -> Result<Vec<CustomMacro>, String> {
    let latex: Vec<char> = latex.chars().collect();
    let pattern: Vec<char> = "\\newcommand".chars().collect();
    let pattern_len = pattern.len();
    let mut pos = 0;
    let mut custom_macros: Vec<CustomMacro> = Vec::new();

    while pos < latex.len().saturating_sub(pattern_len) {
        if latex[pos..pos + pattern_len] == pattern[..] {
            pos += pattern_len;
            // extract the new command name
            let new_command_name: String;
            if latex[pos] != '{' {
                return Err("Expecting { after \\newcommand".to_string());
            }
            pos += 1;
            if latex[pos] != '\\' {
                return Err("Expecting backslash after {".to_string());
            }
            if let Some(right_curly_bracket_pos) = find_matching_right_curly_bracket_char(&latex, pos) {
                new_command_name = latex[pos..right_curly_bracket_pos].iter().collect();
                pos = right_curly_bracket_pos;
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
                pos += 1;
            } else {
                num_of_args = 0;
            }

            // check if there is a default value for the first argument
            let default_value: Option<String>;
            if latex[pos] == '[' {
                pos += 1;
                if let Some(right_square_bracket) = latex[pos..].iter().position(|&c| c == ']') {
                    default_value = Some(latex[pos..pos + right_square_bracket].iter().collect::<String>());
                    pos += right_square_bracket;
                } else {
                    return Err("Unmatched square brackets".to_string());
                }
                pos += 1;
            } else {
                default_value = None;
            }

            // extract the definition
            let definition: String;
            if latex[pos] != '{' {
                return Err("Expecting { before the definition".to_string());
            }
            pos += 1;
            if let Some(right_curly_bracket_pos) = find_matching_right_curly_bracket_char(&latex, pos) {
                definition = latex[pos..right_curly_bracket_pos].iter().collect();
                pos = right_curly_bracket_pos;
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

    Ok(custom_macros)
}

fn construct_custom_macro(
    new_command_name: String,
    num_of_args: usize,
    default_value: Option<String>,
    definition: String,
) -> Result<CustomMacro, String> {
    let command_type: CommandType;
    let implementation: Box<dyn Fn(&Vec<Vec<TexToken>>) -> Result<Vec<TexToken>, String>>;

    if let Some(default_value) = default_value {
        // default value provided, so it's an optional unary or optional binary command
        match num_of_args {
            0 => {
                return Err("Default value provided for a command with no arguments".to_string());
            }
            1 => {
                // optional unary command
                command_type = CommandType::OptionalUnary;
                implementation = Box::new(move |args: &Vec<Vec<TexToken>>| {
                    let replaced_string: String;
                    if args.is_empty() {
                        replaced_string = definition.replace("#1", &default_value);
                    } else {
                        replaced_string = definition.replace(
                            "#1",
                            &args[0].iter().map(|token| token.value.clone()).collect::<String>(),
                        );
                    }
                    tokenize(&replaced_string)
                });
            }
            2 => {
                // optional binary command
                command_type = CommandType::OptionalBinary;
                implementation = Box::new(move |args: &Vec<Vec<TexToken>>| {
                    let replaced_string: String;
                    if args.len() == 1 {
                        replaced_string = definition.replace("#1", &default_value).replace(
                            "#2",
                            &args[0].iter().map(|token| token.value.clone()).collect::<String>(),
                        );
                    } else if args.len() == 2 {
                        replaced_string = definition
                            .replace(
                                "#1",
                                &args[0].iter().map(|token| token.value.clone()).collect::<String>(),
                            )
                            .replace(
                                "#2",
                                &args[1].iter().map(|token| token.value.clone()).collect::<String>(),
                            );
                    } else {
                        return Err("Expecting one or two arguments".to_string());
                    }
                    tokenize(&replaced_string)
                });
            }
            _ => {
                return Err("Only unary and binary commands are supported".to_string());
            }
        }
    } else {
        // no default value, it's either a symbol, unary or binary command
        match num_of_args {
            0 => {
                // symbol command
                command_type = CommandType::Symbol;
                implementation = Box::new(move |_| tokenize(&definition));
            }
            1 => {
                // unary command
                command_type = CommandType::Unary;
                implementation = Box::new(move |args: &Vec<Vec<TexToken>>| {
                    let replaced_string = definition.replace(
                        "#1",
                        &args[0].iter().map(|token| token.value.clone()).collect::<String>(),
                    );
                    tokenize(&replaced_string)
                });
            }
            2 => {
                // binary command
                command_type = CommandType::Binary;
                implementation = Box::new(move |args: &Vec<Vec<TexToken>>| {
                    let replaced_string = definition
                        .replace(
                            "#1",
                            &args[0].iter().map(|token| token.value.clone()).collect::<String>(),
                        )
                        .replace(
                            "#2",
                            &args[1].iter().map(|token| token.value.clone()).collect::<String>(),
                        );
                    tokenize(&replaced_string)
                });
            }
            _ => {
                return Err("Only unary and binary commands are supported".to_string());
            }
        }
    }

    Ok(CustomMacro {
        name: new_command_name,
        command_type,
        implementation,
    })
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
            Ok(vec![TexToken {
                token_type: TexTokenType::Command,
                value: r"\mycommandexpanded".to_string(),
            }])
        };
        registry.register_custom_macro(r"\mycommand", CommandType::Symbol, Box::new(implementation));

        assert_eq!(registry.get_command_type(r"\mycommand"), Some(CommandType::Symbol));

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
            Ok(res)
        };
        registry.register_custom_macro(r"\mycommand", CommandType::Unary, Box::new(implementation));

        assert_eq!(registry.get_command_type(r"\mycommand"), Some(CommandType::Unary));

        let tokens = tokenize(r"\mycommand{a}").unwrap();
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(expanded_tokens, tokenize(r"\expanded{a}").unwrap(),);
    }

    #[test]
    fn test_parse_custom_macros_symbol() {
        let macro_string = r"\newcommand{\mycommand}{\expanded}";
        let tex = r"\mycommand";

        let custom_macros = parse_custom_macros(macro_string).unwrap();

        assert_eq!(custom_macros.len(), 1);
        assert_eq!(custom_macros[0].name, "\\mycommand");
        assert_eq!(custom_macros[0].command_type, CommandType::Symbol);
        assert_eq!(
            (custom_macros[0].implementation)(&vec![]).unwrap(),
            tokenize(r"\expanded").unwrap()
        );

        let mut registry = CommandRegistry::new();
        registry.register_custom_macros(custom_macros);
        let tokens = tokenize(tex).unwrap();
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(expanded_tokens, tokenize(r"\expanded").unwrap());
    }

    #[test]
    fn test_parse_custom_macros_unary() {
        let macro_string = r"\newcommand{\mycommand}[1]{\expanded{#1}}";
        let tex = r"\mycommand{a}";

        let custom_macros = parse_custom_macros(macro_string).unwrap();

        assert_eq!(custom_macros.len(), 1);
        assert_eq!(custom_macros[0].name, "\\mycommand");
        assert_eq!(custom_macros[0].command_type, CommandType::Unary);
        assert_eq!(
            (custom_macros[0].implementation)(&vec![tokenize("a").unwrap()]).unwrap(),
            tokenize(r"\expanded{a}").unwrap()
        );

        let mut registry = CommandRegistry::new();
        registry.register_custom_macros(custom_macros);
        let tokens = tokenize(tex).unwrap();
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(expanded_tokens, tokenize(r"\expanded{a}").unwrap());
    }

    #[test]
    fn test_parse_custom_macros_binary() {
        let macro_string = r"\newcommand{\mycommand}[2]{\expanded{#1}\and{#2}}";
        let tex = r"\mycommand{a}{b}";

        let custom_macros = parse_custom_macros(macro_string).unwrap();

        assert_eq!(custom_macros.len(), 1);
        assert_eq!(custom_macros[0].name, "\\mycommand");
        assert_eq!(custom_macros[0].command_type, CommandType::Binary);
        assert_eq!(
            (custom_macros[0].implementation)(&vec![tokenize("a").unwrap(), tokenize("b").unwrap()]).unwrap(),
            tokenize(r"\expanded{a}\and{b}").unwrap()
        );

        let mut registry = CommandRegistry::new();
        registry.register_custom_macros(custom_macros);
        let tokens = tokenize(tex).unwrap();
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(expanded_tokens, tokenize(r"\expanded{a}\and{b}").unwrap());
    }

    #[test]
    fn test_parse_custom_macros_optional_unary() {
        let macro_string = r"\newcommand{\mycommand}[1][default]{\expanded{#1}}";
        let tex = r"\mycommand \mycommand[a]";

        let custom_macros = parse_custom_macros(macro_string).unwrap();

        assert_eq!(custom_macros.len(), 1);
        assert_eq!(custom_macros[0].name, "\\mycommand");
        assert_eq!(custom_macros[0].command_type, CommandType::OptionalUnary);

        let mut registry = CommandRegistry::new();
        registry.register_custom_macros(custom_macros);
        let tokens = tokenize(tex).unwrap();
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(expanded_tokens, tokenize(r"\expanded{default} \expanded{a}").unwrap());
    }

    #[test]
    fn test_parse_custom_macros_optional_binary() {
        let macro_string = r"\newcommand{\mycommand}[2][def]{\expanded{#1}\and{#2}}";
        let tex = r"\mycommand{b} \mycommand[a]{b}";

        let custom_macros = parse_custom_macros(macro_string).unwrap();

        assert_eq!(custom_macros.len(), 1);
        assert_eq!(custom_macros[0].name, "\\mycommand");
        assert_eq!(custom_macros[0].command_type, CommandType::OptionalBinary);

        let mut registry = CommandRegistry::new();
        registry.register_custom_macros(custom_macros);
        let tokens = tokenize(tex).unwrap();
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(
            expanded_tokens,
            tokenize(r"\expanded{def}\and{b} \expanded{a}\and{b}").unwrap()
        );
    }

    #[test]
    fn test_multiple_custom_macros() {
        let macro_string = r"\newcommand{\mysym}{\texttt{sym}}
        \newcommand{\aunary}[1]{\expanded{#1}}
        \newcommand{\abinary}[2]{\expanded{#1}\and{#2}}
        \newcommand{\aoptionalunary}[1][def1]{\expanded{#1}}
        \newcommand{\aoptionalbinary}[2][def2]{\expanded{#1}\and{#2}}";
        let tex = r"\mysym \aunary{a} \abinary{a}{b} \aoptionalunary \aoptionalunary[a] \aoptionalbinary{b} \aoptionalbinary[a]{b}";

        let custom_macros = parse_custom_macros(macro_string).unwrap();

        assert_eq!(custom_macros.len(), 5);

        let mut registry = CommandRegistry::new();
        registry.register_custom_macros(custom_macros);
        let tokens = tokenize(tex).unwrap();
        let expanded_tokens = registry.expand_macros(&tokens).unwrap();
        assert_eq!(expanded_tokens, tokenize(r"\texttt{sym} \expanded{a} \expanded{a}\and{b} \expanded{def1} \expanded{a} \expanded{def2}\and{b} \expanded{a}\and{b}").unwrap());
    }
}
