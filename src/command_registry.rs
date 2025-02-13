pub const UNARY_COMMANDS: &[&'static str] = &[
    // "sqrt",
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
    CustomMacro,
}

#[derive(Default)]
pub struct CommandRegistry {
    custom_macros: Vec<String>,
}

impl CommandRegistry {
    pub(crate) fn new() -> CommandRegistry {
        Self::default()
    }

    pub fn register_custom_macro(&mut self, name: &str, command_type: CommandType) {
        todo!()
    }

    pub fn get_command_type(&self, command_name: &str) -> Option<CommandType> {
        if UNARY_COMMANDS.contains(&command_name) {
            Some(CommandType::Unary)
        } else if BINARY_COMMANDS.contains(&command_name) {
            Some(CommandType::Binary)
        } else if OPTION_BINARY_COMMANDS.contains(&command_name) {
            Some(CommandType::OptionalBinary)
        } else if self.custom_macros.iter().any(|macro_name| macro_name == command_name) {
            Some(CommandType::CustomMacro)
        } else {
            Some(CommandType::Symbol)
        }
    }
}
