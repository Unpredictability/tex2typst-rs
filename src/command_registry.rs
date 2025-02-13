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

pub enum CommandType {
    Symbol,
    Unary,
    Binary,
    OptionalBinary,
}

pub struct CustomMacros {
    pub name: String,
    pub command_type: CommandType,
    pub implementation: Box<dyn Fn(&[TexToken]) -> Vec<TexToken>>,
}

#[derive(Default)]
pub struct CommandRegistry {
    custom_macros: Vec<CustomMacros>,
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
        self.custom_macros.push(CustomMacros {
            name: name.to_string(),
            command_type,
            implementation,
        });
    }

    pub fn get_command_type(&self, command_name: &str) -> Option<CommandType> {
        if UNARY_COMMANDS.contains(&command_name) {
            Some(CommandType::Unary)
        } else if BINARY_COMMANDS.contains(&command_name) {
            Some(CommandType::Binary)
        } else if OPTION_BINARY_COMMANDS.contains(&command_name) {
            Some(CommandType::OptionalBinary)
        } else if self
            .custom_macros
            .iter()
            .any(|custom_macro| custom_macro.name == command_name)
        {
            // Custom macro handling
            todo!("Implement custom macro handling");
        } else {
            Some(CommandType::Symbol)
        }
    }
}
