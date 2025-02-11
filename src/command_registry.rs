use crate::definitions::TexNode;

enum CommandType {
    Symbol,
    Unary,
    Binary,
    OptionalBinary,
    CustomMacro,
}

pub struct MacroRegistry {
    macros: std::collections::HashMap<String, CommandType>,
}

impl MacroRegistry {
    pub(crate) fn new() -> MacroRegistry {
        todo!()
    }
}
