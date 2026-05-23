/*
 * Nuva OS - SystemLibrary - Lang
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


/// Token Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // ===== Literal =====
    /// Integer
    Integer,
    /// Float
    Float,
    /// String
    String,
    /// Character
    Char,
    /// boolean
    Bool,
    
    // ===== IdentifiersumKeyword =====
    /// Identifier
    Identifier,
    /// Keyword
    Keyword,
    
    // ===== Operator =====
    /// Plus
    Plus,
    /// Minus
    Minus,
    /// Multiply
    Star,
    /// Divide
    Slash,
    /// Modulo
    Percent,
    /// Assignment
    Assign,
    /// Equal
    Equal,
    /// Not Equal
    NotEqual,
    /// Less Than
    Less,
    /// Less ThanEqual
    LessEqual,
    /// Greater Than
    Greater,
    /// Greater ThanEqual
    GreaterEqual,
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Logical NOT
    Not,
    /// Bitwith
    BitAnd,
    /// Bitor
    BitOr,
    /// Bitwise XOR
    BitXor,
    /// Bitnon
    BitNot,
    /// Left Shift
    LeftShift,
    /// Right Shift
    RightShift,
    /// PlusAssignment
    PlusAssign,
    /// MinusAssignment
    MinusAssign,
    /// MultiplyAssignment
    StarAssign,
    /// DivideAssignment
    SlashAssign,
    /// Pipeline operator |>
    Pipeline,
    
    // ===== Delimiter =====
    /// Left Parenthesis
    LeftParen,
    /// Right Parenthesis
    RightParen,
    /// Left Bracket
    LeftBracket,
    /// Right Bracket
    RightBracket,
    /// Left Brace
    LeftBrace,
    /// Right Brace
    RightBrace,
    /// Comma
    Comma,
    /// Semicolon
    Semicolon,
    /// Colon
    Colon,
    /// Point
    Dot,
    /// Arrow
    Arrow,
    /// Double Colon
    DoubleColon,
    
    // ===== Special =====
    /// Error
    Error,
    /// FileEnd
    Eof,
}

/// Keyword
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // ===== Declaration =====
    /// let
    Let,
    /// var
    Var,
    /// const
    Const,
    /// fn
    Fn,
    /// struct
    Struct,
    /// enum
    Enum,
    /// trait
    Trait,
    /// impl
    Impl,
    /// type
    Type,
    
    // ===== ControlFlow =====
    /// if
    If,
    /// else
    Else,
    /// match
    Match,
    /// while
    While,
    /// for
    For,
    /// in
    In,
    /// loop
    Loop,
    /// break
    Break,
    /// continue
    Continue,
    /// return
    Return,
    
    // ===== Type =====
    /// i8, i16, i32, i64, i128
    Int,
    /// u8, u16, u32, u64, u128
    Uint,
    /// f32, f64
    Float,
    /// bool
    Bool,
    /// char
    Char,
    /// str
    Str,
    
    // ===== Modifier =====
    /// pub
    Pub,
    /// priv
    Priv,
    /// mut
    Mut,
    /// static
    Static,
    /// async
    Async,
    /// await
    Await,

    // ===== Declarative =====
    /// component (declarative UI)
    Component,
    /// signal (reactive data source)
    Signal,
    /// effect (reactive side effect)
    Effect,
    /// reactive (reactive wrapper type)
    Reactive,
    /// resource (declarative resource management)
    Resource,
    /// with (resource cleanup / scope)
    With,

    // ===== Other =====
    /// use
    Use,
    /// mod
    Mod,
    /// self
    Self_,
    /// super
    Super,
    /// true
    True,
    /// false
    False,
    /// None
    None,
    /// Some
    Some,
}

/// Token
#[derive(Debug, Clone)]
pub struct Token {
    /// Token Type
    pub token_type: TokenType,
    /// Keyword (ifisKeyword)
    pub keyword: Option<Keyword>,
    /// Literalvalue
    pub value: TokenValue,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Length
    pub length: u32,
}

/// Token value
#[derive(Debug, Clone)]
pub enum TokenValue {
    /// nonevalue
    None,
    /// Integer
    Integer(i64),
    /// Float
    Float(f64),
    /// String
    String(&'static str),
    /// Character
    Char(char),
    /// boolean
    Bool(bool),
    /// Identifier
    Identifier(&'static str),
}

impl Token {
    /// Createnew Token
    pub fn new(token_type: TokenType, line: u32, column: u32) -> Self {
        Token {
            token_type,
            keyword: None,
            value: TokenValue::None,
            line,
            column,
            length: 1,
        }
    }
    
    /// CreateInteger Token
    pub fn integer(value: i64, line: u32, column: u32, length: u32) -> Self {
        Token {
            token_type: TokenType::Integer,
            keyword: None,
            value: TokenValue::Integer(value),
            line,
            column,
            length,
        }
    }
    
    /// CreateFloat Token
    pub fn float(value: f64, line: u32, column: u32, length: u32) -> Self {
        Token {
            token_type: TokenType::Float,
            keyword: None,
            value: TokenValue::Float(value),
            line,
            column,
            length,
        }
    }
    
    /// CreateString Token
    pub fn string(value: &'static str, line: u32, column: u32, length: u32) -> Self {
        Token {
            token_type: TokenType::String,
            keyword: None,
            value: TokenValue::String(value),
            line,
            column,
            length,
        }
    }
    
    /// CreateIdentifier Token
    pub fn identifier(value: &'static str, line: u32, column: u32, length: u32) -> Self {
        Token {
            token_type: TokenType::Identifier,
            keyword: None,
            value: TokenValue::Identifier(value),
            line,
            column,
            length,
        }
    }
    
    /// CreateKeyword Token
    pub fn keyword(keyword: Keyword, line: u32, column: u32, length: u32) -> Self {
        Token {
            token_type: TokenType::Keyword,
            keyword: Some(keyword),
            value: TokenValue::None,
            line,
            column,
            length,
        }
    }
    
    /// CreateError Token
    pub fn error(message: &'static str, line: u32, column: u32) -> Self {
        Token {
            token_type: TokenType::Error,
            keyword: None,
            value: TokenValue::String(message),
            line,
            column,
            length: 1,
        }
    }
}