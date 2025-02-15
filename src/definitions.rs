use std::collections::HashMap;

// Control: {, }, _, ^, &, \
// Element: [, ],
#[derive(Debug, PartialEq, Clone)]
pub enum TexTokenType {
    Element,
    Command,
    Text,
    Comment,
    Space,
    Newline,
    Control,
    Unknown,
    NoBreakSpace,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TexToken {
    pub token_type: TexTokenType,
    pub value: String,
}

impl TexToken {
    pub fn new(token_type: TexTokenType, value: String) -> Self {
        TexToken { token_type, value }
    }
}

type TexArrayData = Vec<Vec<TexNode>>;

// element: 0-9, a-z, A-Z, punctuations such as +-/*,:; etc.
// symbol: LaTeX macro with no parameter. e.g. \sin \cos \int \sum
// unaryFunc: LaTeX macro with 1 parameter. e.g. \sqrt{3} \log{x} \exp{x}
// binaryFunc: LaTeX macro with 2 parameters. e.g. \frac{1}{2}
// text: text enclosed by braces. e.g. \text{hello world}
// empty: special type when something is empty. e.g. the base of _{a} or ^{a}
// whitespace: space, tab, newline
#[derive(Debug, PartialEq, Clone)]
pub enum TexNodeType {
    Element,
    Text,
    Comment,
    Whitespace,
    Control,
    Ordgroup,
    SupSub,
    UnaryFunc,
    BinaryFunc,
    OptionBinaryFunc,
    Leftright,
    BeginEnd,
    Symbol,
    Empty,
    UnknownMacro,
    NoBreakSpace,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TexNode {
    pub node_type: TexNodeType,
    pub content: String,
    pub args: Option<Vec<TexNode>>,   // when node_type is Command, args is the parameters
    pub data: Option<Box<TexNodeData>>,  // for stuff like begin-end, array, etc.
}

#[derive(Debug, PartialEq, Clone)]
pub enum TexNodeData {
    Supsub(TexSupsubData),
    Array(TexArrayData),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TexSupsubData {
    pub base: TexNode,
    pub sup: Option<TexNode>,
    pub sub: Option<TexNode>,
}

impl TexNode {
    pub fn new(
        node_type: TexNodeType,
        content: String,
        args: Option<Vec<TexNode>>,
        data: Option<Box<TexNodeData>>,
    ) -> Self {
        TexNode {
            node_type,
            content,
            args,
            data,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypstTokenType {
    Symbol,
    Element,
    Text,
    Comment,
    Control,
}

#[derive(Debug, PartialEq)]
pub enum TypstNodeType {
    Atom,
    Symbol,
    Text,
    Comment,
    Whitespace,
    Empty,
    Group,
    Supsub,
    FuncCall,
    Fraction,
    Align,
    Matrix,
    Unknown,
    NoBreakSpace,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypstToken {
    pub token_type: TypstTokenType,
    pub value: String,
}

impl TypstToken {
    pub fn new(token_type: TypstTokenType, value: String) -> Self {
        TypstToken { token_type, value }
    }

    pub fn to_string(&self) -> String {
        match self.token_type {
            TypstTokenType::Text => format!("\"{}\"", self.value),
            TypstTokenType::Comment => format!("//{}", self.value),
            _ => self.value.clone(),
        }
    }
}

#[derive(Debug)]
pub struct TypstNode {
    pub node_type: TypstNodeType,
    pub content: String,
    pub args: Option<Vec<TypstNode>>,
    pub data: Option<Box<TypstNodeData>>,
    pub options: Option<TypstNamedParams>,
}

impl TypstNode {
    pub fn new(
        node_type: TypstNodeType,
        content: String,
        args: Option<Vec<TypstNode>>,
        data: Option<Box<TypstNodeData>>,
    ) -> Self {
        TypstNode {
            node_type,
            content,
            args,
            data,
            options: None,
        }
    }

    pub fn set_options(&mut self, options: TypstNamedParams) {
        self.options = Some(options);
    }
}

impl PartialEq for TypstNode {
    fn eq(&self, other: &TypstNode) -> bool {
        self.node_type == other.node_type && self.content == other.content
    }
}

pub type TypstNamedParams = HashMap<String, String>;

#[derive(Debug, PartialEq)]
pub enum TypstNodeData {
    Supsub(TypstSupsubData),
    Array(TypstArrayData),
}

#[derive(Debug, PartialEq)]
pub struct TypstSupsubData {
    pub base: TypstNode,
    pub sup: Option<TypstNode>,
    pub sub: Option<TypstNode>,
}

type TypstArrayData = Vec<Vec<TypstNode>>;
