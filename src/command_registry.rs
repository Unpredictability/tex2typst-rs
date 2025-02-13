use std::collections::HashMap;
use crate::definitions::TexToken;

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
    pub implementation: Box<dyn Fn(&[TexToken]) -> Vec<TexToken>>,
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
        implementation: Box<dyn Fn(&[TexToken]) -> Vec<TexToken>>,
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
            Some(CommandType::Symbol)
        }
    }
}

pub fn parse_custom_macros(latex: &str) -> Result<CustomMacro, String> {
    todo!("idealy, it should accept raw latex \newcommand definitions, but may be hard to parse");
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
        assert_eq!(tokens, vec![TexToken {
            token_type: TexTokenType::Command,
            value: r"\alpha".to_string(),
        }]);
    }

    #[test]
    fn test_command_registry_simple_symbol() {
        let mut registry = CommandRegistry::new();

        let implementation = |tokens: &[TexToken]| vec![TexToken {
            token_type: TexTokenType::Command,
            value: r"\mycommandexpanded".to_string(),
        }];
        registry.register_custom_macro("mycommand", CommandType::Symbol, Box::new(implementation));

        assert_eq!(registry.get_command_type("text"), Some(CommandType::Unary));
        assert_eq!(registry.get_command_type("frac"), Some(CommandType::Binary));
        assert_eq!(registry.get_command_type("sqrt"), Some(CommandType::OptionalBinary));
        assert_eq!(registry.get_command_type("mycommand"), Some(CommandType::Symbol));
    }
}
