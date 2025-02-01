use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TexTokenType {
    Element,
    Command,
    Text,
    Comment,
    Space,
    Newline,
    Control,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TexToken {
    pub(crate) token_type: TexTokenType,
    pub(crate) value: String,
}

impl TexToken {
    pub(crate) fn new(token_type: TexTokenType, value: String) -> Self {
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
pub(crate) enum TexNodeType {
    Element,
    Text,
    Comment,
    Whitespace,
    Control,
    Ordgroup,
    SupSub,
    UnaryFunc,
    BinaryFunc,
    Leftright,
    BeginEnd,
    Symbol,
    Empty,
    UnknownMacro,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TexNode {
    pub(crate) node_type: TexNodeType,
    pub(crate) content: String,
    pub(crate) args: Option<Vec<TexNode>>,
    pub(crate) data: Option<Box<TexNodeData>>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TexNodeData {
    Sqrt(TexNode),
    Supsub(TexSupsubData),
    Array(TexArrayData),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TexSupsubData {
    pub(crate) base: TexNode,
    pub(crate) sup: Option<TexNode>,
    pub(crate) sub: Option<TexNode>,
}

impl TexNode {
    pub(crate) fn new(
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
pub(crate) enum TypstTokenType {
    Symbol,
    Element,
    Text,
    Comment,
    Control,
}

#[derive(Debug, PartialEq)]
pub(crate) enum TypstNodeType {
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
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TypstToken {
    pub(crate) token_type: TypstTokenType,
    pub(crate) value: String,
}

impl TypstToken {
    pub(crate) fn new(token_type: TypstTokenType, value: String) -> Self {
        TypstToken { token_type, value }
    }

    pub(crate) fn to_string(&self) -> String {
        match self.token_type {
            TypstTokenType::Text => format!("\"{}\"", self.value),
            TypstTokenType::Comment => format!("//{}", self.value),
            _ => self.value.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TypstNode {
    pub(crate) node_type: TypstNodeType,
    pub(crate) content: String,
    pub(crate) args: Option<Vec<TypstNode>>,
    pub(crate) data: Option<Box<TypstNodeData>>,
    pub(crate) options: Option<TypstNamedParams>,
}

impl TypstNode {
    pub(crate) fn new(
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

    pub(crate) fn set_options(&mut self, options: TypstNamedParams) {
        self.options = Some(options);
    }
}

impl PartialEq for TypstNode {
    fn eq(&self, other: &TypstNode) -> bool {
        self.node_type == other.node_type && self.content == other.content
    }
}

pub(crate) type TypstNamedParams = HashMap<String, String>;

#[derive(Debug, PartialEq)]
pub(crate) enum TypstNodeData {
    Supsub(TypstSupsubData),
    Array(TypstArrayData),
}

#[derive(Debug, PartialEq)]
pub(crate) struct TypstSupsubData {
    pub(crate) base: TypstNode,
    pub(crate) sup: Option<TypstNode>,
    pub(crate) sub: Option<TypstNode>,
}

type TypstArrayData = Vec<Vec<TypstNode>>;
