
#![feature(exclusive_range_pattern)]

use std::fs::File;
use std::io::{Error, Read};
use std::env;
use std::string::String;
use std::str::Chars;
use std::borrow::Borrow;
use array_init;


// 通过第一个字符判断token类型
#[derive(PartialEq, Clone)]
#[warn(dead_code)]
enum NodeType {
    NotAllow,
    Num,
    IdentOrKeyword,
    SingleCharSymbol,
    MultiCharSymbol,
}

#[derive(Clone)]
struct DfaNode {
    ch: char,

    node_type: NodeType,
    childs: Vec<Box<DfaNode>>,

    could_be_end: bool,
    token: Option<Token>,

    // could be a part of
    ident: bool,
    symbol: bool,
    line_end: bool,
    space: bool,
    num_float: bool,
    num_hex: bool,
    num_bin: bool,
    num_int: bool,
    is_delimit: bool,
}

impl DfaNode {
    fn new() -> DfaNode {
        DfaNode {
            ch: '\0',
            could_be_end: true,
            token: None,
            ident: false,
            symbol: false,
            line_end: false,
            space: false,
            num_float: false,
            num_hex: false,
            node_type: NodeType::NotAllow,
            childs: vec![],
            num_int: false,
            is_delimit: false,
            num_bin: false
        }
    }
}

#[warn(dead_code)]
#[derive(Clone, Debug)]
enum Token {
    KwFn,
    KwInt,
    KwFloat,
    KwLet,
    Comma,    // ,
    Equal,    // =
    Plus,     // +
    Minus,    // -
    ParenL,    // (
    ParenR,    // )
    BraceL,    // {
    BraceR,    // }
    NextLine,  // \n
    EqualEqual, // ==
    Gt,         // >
    Lt,         // <
    GtEqual,    // >=
    LtEqual,    // <=
    Star,       // *
    Slash,      // /
    PlusEqaul,  // +=
    MinusEqual, // -=
    StarEqual,  // *=
    SlashEqual,  // /=

    LitStr(String),
    LitInt(String, i64),
    LitFloat(String, f64),
    Identifier(String),
    // EOF,
}

#[warn(dead_code)]
impl Token {
    fn to_str(&self) -> &str{
        match self {
            Token::KwInt => "int",
            Token::KwFn => "fn",
            Token::ParenL => "(",
            Token::ParenR => ")",
            Token::BraceL => "{",
            Token::BraceR => "}",
            Token::Comma => ",",
            Token::Identifier(name) => name,
            Token::KwLet => "let",
            Token::Equal=> "=",
            Token::KwFloat=> "float",
            Token::LitStr(v) => v.as_str(),
            Token::Plus => "+",
            Token::LitFloat(v, _) => v.as_str(),
            Token::LitInt(v, _) => v.as_str(),
            Token::NextLine => "\n",
            Token::Lt => "<",
            Token::Gt => ">",
            Token::Minus => "-",
            Token::EqualEqual => "==",
            Token::GtEqual => ">=",
            Token::LtEqual => "<=",
            Token::Star => "*",
            Token::Slash => "/",
            Token::PlusEqaul => "+=",
            Token::MinusEqual => "-+",
            Token::StarEqual => "*=",
            Token::SlashEqual => "/=",
        }
    }
}

const ASCII_COUNT: usize = 128;
struct Lexer<'a> {
    // buf: String,
    iter: Chars<'a>,
    root: [DfaNode; ASCII_COUNT],
    tokens: Vec<Token>,

    prev_ch: Option<char>,
    literal: String
}

impl Lexer<'_> {
    fn new(buf: &String) -> Lexer {
        let chars_itr = buf.chars();
        Lexer {
            iter: chars_itr,
            root: array_init::array_init(|i| {
                DfaNode {
                    ch: char::from(i as u8),
                    could_be_end: true,
                    token: None,
                    ident: false,
                    symbol: false,
                    line_end: false,
                    space: false,
                    num_float: false,
                    num_hex: false,
                    node_type: NodeType::NotAllow,
                    childs: vec![],
                    num_int: false,
                    is_delimit: false,
                    num_bin: false
                }
            }),
            tokens: vec![],
            prev_ch: None,
            literal: "".to_string()
        }
    }

    fn get_ch(&mut self) -> Option<char> {
        loop {
            let ch = self.iter.next();
            if None == ch {
                return None;
            }
            if self.prev_ch == ch && ch.unwrap() == ' ' && ch.unwrap() == '\n' {
                continue;
            }
            self.prev_ch = ch;
            return ch;
        }
    }

    fn eat_identifier(&mut self) -> Option<DfaNode> {
        loop {
            let ch_option = self.get_ch();
            if ch_option == None {
                return None;
            }
            let ch = ch_option.unwrap();
            let node = &self.root[ch as usize];
            if node.ident {
                self.literal.push(ch);
            } else {
                return Option::from(node.clone());
            }
        }
    }

    fn eat_number(&mut self) -> Option<DfaNode> {
        loop {
            let ch_option = self.get_ch();
            if ch_option == None {
                return None;
            }
            let ch = ch_option.unwrap();
            let node = &self.root[ch as usize];
            if node.num_int {
                self.literal.push(ch);
            } else {
                return Option::from(node.clone());
            }
        }
    }

    fn lex_node(&mut self, node: &DfaNode) {
        // next char
        let next_ch_option = self.get_ch();
        if next_ch_option == None {
            return;
        }
        let next_ch = next_ch_option.unwrap();
        let next_node = &self.root[node.ch as usize];

        match node.node_type {
            NodeType::MultiCharSymbol => {
                for child in &node.childs {
                    if child.ch == next_ch {
                        assert!(child.could_be_end);
                        return self.tokens.push(child.token.clone().unwrap());
                    }
                }
                return;
            }
            NodeType::IdentOrKeyword => {
                self.literal.push(node.ch);
                for child in &node.childs {
                    if child.ch == next_ch {
                        return self.lex_node(child);
                    }
                }
                if next_node.is_delimit {
                    self.tokens.push(node.token.clone().unwrap());
                    return self.lex_node(node);
                }

                if !next_node.ident {
                    print!("unidentifiled char {}.", next_ch);
                    return;
                }
                self.literal.push(next_ch);
                let delimit_node = self.eat_identifier();
                self.tokens.push(Token::Identifier(self.literal.clone()));

                match delimit_node {
                    Some(n) => {
                        return self.lex_node(n.borrow());
                    }
                    None => {return;}
                }

            }
            NodeType::Num => {
                self.literal.push(node.ch);

                let delimit_node = self.eat_number();
                self.tokens.push(
                    Token::LitInt(
                        self.literal.clone(),
                        self.literal.parse().unwrap()
                    ));

                match delimit_node {
                    Some(n) => {
                        return self.lex_node(n.borrow());
                    }
                    None => {return;}
                }
            }
            _ => {}
        }
    }

    fn lex(&mut self) {
        self.build_dfa_tree();
        loop {
            let ch = self.get_ch();
            if ch == None {
                return;
            }
            let node = self.root[ch.unwrap() as usize].clone();

            if node.node_type == NodeType::SingleCharSymbol
            {
                self.tokens.push(node.token.clone().unwrap());
                continue;
            }
            self.lex_node(&node);
        }
    }

    fn build_dfa_tree(&mut self) {
//        const ASCII_COUNT: usize = 128;
//        let mut root: [DfaNode; ASCII_COUNT] = array_init::array_init(|i| {
//            DfaNode{ ch: char::from(i as u8), could_be_end: true, node_type: NodeType::Space, childs: vec![] }
//        });

        for ch in 'A' as usize .. 'z' as usize {
            let node = &mut self.root[ch];
            node.node_type = NodeType::IdentOrKeyword;
            node.ident = true;
            // root[ch].could_be_end = false;
        }
        for ch in '0' as usize .. '9' as usize {
            self.root[ch].node_type = NodeType::Num;
            // root[ch as u8].could_be_end = false;
        }
    }
}

//const tokens: [Token] = [
//    KwFn,
//    Identifier("add"),
//    ParenL,
//    Identifier("a"),
//    KwInt,
//    Comma,
//    Identifier("b"),
//    KwInt,
//    ParenR,
//    KwInt,
//    BraceL,
//    Identifier("a"),
//    Plus,
//    Identifier("b"),
//    BraceR
//];


//impl Copy for DfaNode {
//    // fn copy() -> DfaNode { DfaNode{ ch:'\0', could_be_end: false, child: vec![] } }
//}

// 数字
// 标识符
// 空格
// 符号
//


fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: \n\troc <source file>");
        return Ok(());
    }
    let path = &args[1];
    let mut input = File::open(path)?;
    let mut buf = String::new();
    input.read_to_string(&mut buf).unwrap();

    let mut lexer = Lexer::new(&buf);
    lexer.lex();

    Ok(())
}
