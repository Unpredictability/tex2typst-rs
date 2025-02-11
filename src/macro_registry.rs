use crate::definitions::TexNode;


enum Macro {
    SymbolMacro(SymbolMacro),
    UnaryMacro(UnaryMacro),
}

struct SymbolMacro {
    name: String,
    definition: fn() -> TexNode,
}

struct UnaryMacro {
    name: String,
    num_params: usize,
    definition: fn(Vec<TexNode>) -> TexNode, // the length of the Vec must be equal to num_params
}

impl UnaryMacro {
    pub fn new(name: String, num_params: usize, definition: fn(Vec<TexNode>) -> TexNode) -> Macro {
        Macro {
            name,
            num_params,
            definition,
        }
    }

    pub fn expand(&self, args: Vec<TexNode>) -> TexNode {
        (self.definition)(args)
    }
}

pub struct MacroRegistry {
    macros: std::collections::HashMap<String, Macro>,
}

impl MacroRegistry {
    pub(crate) fn new() -> MacroRegistry {
        todo!()
    }
}
