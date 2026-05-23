/* * Nuva OS - Tools - Compiler
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

// ! Nuva languagelanguageLexical Analysisdevice

/// Token Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TokenKind {
    // Literal
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    BoolLiteral,
    NilLiteral,
    
    // IdentifiersumKeyword
    Identifier,
    
    // Keyword
    KwFunc,
    KwVar,
    KwLet,
    KwClass,
    KwStruct,
    KwEnum,
    KwProtocol,
    KwExtension,
    KwIf,
    KwElse,
    KwSwitch,
    KwCase,
    KwDefault,
    KwFor,
 KwWhile,
    KwDo,
    KwBreak,
    KwContinue,
    KwReturn,
    KwThrow,
    KwTry,
    KwCatch,
    KwFinally,
    KwImport,
    KwExport,
    KwPublic,
    KwPrivate,
    KwInternal,
    KwStatic,
    KwMut,
    KwConst,
    KwAsync,
    KwAwait,
    KwSelf,
    KwSuper,
    KwInit,
    KwDeinit,
    KwGet,
    KwSet,
    KwIn,
    KwWhere,
    KwGuard,
    KwDefer,
    
    // Operator
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          ///
    Percent,        // %
    Amp,            // &
    Pipe,           // |
    Caret,          // ^
    Tilde,          // ~
    Exclaim,        // !
    Eq,             // =
    EqEq,           // ==
    BangEq,         // !=
    Lt,             // <
    Gt,             // >
    LtEq,           // <=
    GtEq,           // >=
    LtLt,           // <<
    GtGt,           // >>
    AmpAmp,         // &&
    PipePipe,       // ||
    PlusEq,         // +=
    MinusEq,        // -=
    StarEq,         // *=
    SlashEq,        ///=
    PercentEq,      // %=
    AmpEq,          // &=
    PipeEq,         // |=
    CaretEq,        // ^=
    LtLtEq,         // <<=
    GtGtEq,         // >>=
    Arrow,          // ->
    FatArrow,       // =>
    Range,          // ..
    RangeInclusive, // ...
    NilCoalesce,    // ??
    
    // Delimiter
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Comma,          // ,
    Semicolon,      // ;
    Colon,          // :
    DoubleColon,    // ::
    Dot,            // .
    Hash,           // #
    At,             // @
    Dollar,         // $
    Question,       // ?
    
    // Special
    Eof,
    Error,
}

/// Token
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: [u8; 256],
    pub lexeme_len: u8,
    pub line: u32,
    pub column: u32,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: &[u8], line: u32, column: u32) -> Self {
        let mut lexeme_buf = [0u8; 256];
        let len = lexeme.len().min(255);
        lexeme_buf[..len].copy_from_slice(&lexeme[..len]);
        
        Self {
            kind,
            lexeme: lexeme_buf,
            lexeme_len: len as u8,
            line,
            column,
        }
    }

    pub fn lexeme(&self) -> &[u8] {
        &self.lexeme[..self.lexeme_len as usize]
    }
}

/// Lexical Analyzer
pub struct Lexer {
    source: [u8; 65536],
    source_len: usize,
    pos: usize,
    line: u32,
    column: u32,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            source: [0; 65536],
            source_len: 0,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn init(&mut self, source: &[u8]) {
        let len = source.len().min(65535);
        self.source[..len].copy_from_slice(&source[..len]);
        self.source_len = len;
        self.pos = 0;
        self.line = 1;
        self.column = 1;
    }

    /// GetNext Token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        
        if self.is_at_end() {
            return Token::new(TokenKind::Eof, b"", self.line, self.column);
        }
        
        let start_line = self.line;
        let start_column = self.column;
        let c = self.advance();
        
        match c {
            // formCharacter Token
            b'(' => Token::new(TokenKind::LParen, b"(", start_line, start_column),
            b')' => Token::new(TokenKind::RParen, b")", start_line, start_column),
            b'{' => Token::new(TokenKind::LBrace, b"{", start_line, start_column),
            b'}' => Token::new(TokenKind::RBrace, b"}", start_line, start_column),
            b'[' => Token::new(TokenKind::LBracket, b"[", start_line, start_column),
            b']' => Token::new(TokenKind::RBracket, b"]", start_line, start_column),
            b',' => Token::new(TokenKind::Comma, b",", start_line, start_column),
            b';' => Token::new(TokenKind::Semicolon, b";", start_line, start_column),
            b'#' => Token::new(TokenKind::Hash, b"#", start_line, start_column),
            b'@' => Token::new(TokenKind::At, b"@", start_line, start_column),
            b'$' => Token::new(TokenKind::Dollar, b"$", start_line, start_column),
            
            // cancanismanyCharacter Operator
            b'+' => self.match_or_assign(b'+', TokenKind::Plus, TokenKind::PlusEq, start_line, start_column),
            b'-' => self.match_minus(start_line, start_column),
            b'*' => self.match_or_assign(b'*', TokenKind::Star, TokenKind::StarEq, start_line, start_column),
            b'/' => self.match_or_assign(b'/', TokenKind::Slash, TokenKind::SlashEq, start_line, start_column),
            b'%' => self.match_or_assign(b'%', TokenKind::Percent, TokenKind::PercentEq, start_line, start_column),
            b'&' => self.match_amp(start_line, start_column),
            b'|' => self.match_pipe(start_line, start_column),
            b'^' => self.match_or_assign(b'^', TokenKind::Caret, TokenKind::CaretEq, start_line, start_column),
            b'~' => Token::new(TokenKind::Tilde, b"~", start_line, start_column),
            b'!' => self.match_exclaim(start_line, start_column),
            b'=' => self.match_eq(start_line, start_column),
            b'<' => self.match_lt(start_line, start_column),
            b'>' => self.match_gt(start_line, start_column),
            b':' => self.match_colon(start_line, start_column),
            b'.' => self.match_dot(start_line, start_column),
            b'?' => self.match_question(start_line, start_column),
            
            // StringLiteral
            b'"' => self.string_literal(start_line, start_column),
            
            // numberWordLiteral
            b'0'..=b'9' => self.number_literal(c, start_line, start_column),
            
            // IdentifiersumKeyword
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier_or_keyword(c, start_line, start_column),
            
            _ => Token::new(TokenKind::Error, &[c], start_line, start_column),
        }
    }

    fn advance(&mut self) -> u8 {
        let c = self.source[self.pos];
        self.pos += 1;
        if c == b'
' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() { 0 } else { self.source[self.pos] }
    }

    fn peek_next(&self) -> u8 {
        if self.pos + 1 >= self.source_len { 0 } else { self.source[self.pos + 1] }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source_len
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            let c = self.peek();
            match c {
                b' ' | b'\t' | b'\r' | b'
' => {
                    self.advance();
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        // formrowComment
                        while !self.is_at_end() && self.peek() != b'
' {
                            self.advance();
                        }
                    } else if self.peek_next() == b'*' {
                        // manyrowComment
                        self.advance(); ///
                        self.advance(); // *
                        while !self.is_at_end() {
                            if self.peek() == b'*' && self.peek_next() == b'/' {
                                self.advance();
                                self.advance();
                                break;
                            }
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn match_or_assign(&mut self, ch: u8, single: TokenKind, assign: TokenKind, line: u32, col: u32) -> Token {
        if self.peek() == b'=' {
            self.advance();
            let lexeme = &[ch, b'='];
            Token::new(assign, lexeme, line, col)
        } else {
            Token::new(single, &[ch], line, col)
        }
    }

    fn match_minus(&mut self, line: u32, col: u32) -> Token {
        match self.peek() {
            b'=' => {
                self.advance();
                Token::new(TokenKind::MinusEq, b"-=", line, col)
            }
            b'>' => {
                self.advance();
                Token::new(TokenKind::Arrow, b"->", line, col)
            }
            _ => Token::new(TokenKind::Minus, b"-", line, col),
        }
    }

    fn match_amp(&mut self, line: u32, col: u32) -> Token {
        match self.peek() {
            b'=' => {
                self.advance();
                Token::new(TokenKind::AmpEq, b"&=", line, col)
            }
            b'&' => {
                self.advance();
                Token::new(TokenKind::AmpAmp, b"&&", line, col)
            }
            _ => Token::new(TokenKind::Amp, b"&", line, col),
        }
    }

    fn match_pipe(&mut self, line: u32, col: u32) -> Token {
        match self.peek() {
            b'=' => {
                self.advance();
                Token::new(TokenKind::PipeEq, b"|=", line, col)
            }
            b'|' => {
                self.advance();
                Token::new(TokenKind::PipePipe, b"||", line, col)
            }
            _ => Token::new(TokenKind::Pipe, b"|", line, col),
        }
    }

    fn match_exclaim(&mut self, line: u32, col: u32) -> Token {
        if self.peek() == b'=' {
            self.advance();
            Token::new(TokenKind::BangEq, b"!=", line, col)
        } else {
            Token::new(TokenKind::Exclaim, b"!", line, col)
        }
    }

    fn match_eq(&mut self, line: u32, col: u32) -> Token {
        match self.peek() {
            b'=' => {
                self.advance();
                Token::new(TokenKind::EqEq, b"==", line, col)
            }
            b'>' => {
                self.advance();
                Token::new(TokenKind::FatArrow, b"=>", line, col)
            }
            _ => Token::new(TokenKind::Eq, b"=", line, col),
        }
    }

    fn match_lt(&mut self, line: u32, col: u32) -> Token {
        match self.peek() {
            b'=' => {
                self.advance();
                Token::new(TokenKind::LtEq, b"<=", line, col)
            }
            b'<' => {
                self.advance();
                if self.peek() == b'=' {
                    self.advance();
                    Token::new(TokenKind::LtLtEq, b"<<=", line, col)
                } else {
                    Token::new(TokenKind::LtLt, b"<<", line, col)
                }
            }
            _ => Token::new(TokenKind::Lt, b"<", line, col),
        }
    }

    fn match_gt(&mut self, line: u32, col: u32) -> Token {
        match self.peek() {
            b'=' => {
                self.advance();
                Token::new(TokenKind::GtEq, b">=", line, col)
            }
            b'>' => {
                self.advance();
                if self.peek() == b'=' {
                    self.advance();
                    Token::new(TokenKind::GtGtEq, b">>=", line, col)
                } else {
                    Token::new(TokenKind::GtGt, b">>", line, col)
                }
            }
            _ => Token::new(TokenKind::Gt, b">", line, col),
        }
    }

    fn match_colon(&mut self, line: u32, col: u32) -> Token {
        if self.peek() == b':' {
            self.advance();
            Token::new(TokenKind::DoubleColon, b"::", line, col)
        } else {
            Token::new(TokenKind::Colon, b":", line, col)
        }
    }

    fn match_dot(&mut self, line: u32, col: u32) -> Token {
        if self.peek() == b'.' {
            self.advance();
            if self.peek() == b'.' {
                self.advance();
                Token::new(TokenKind::RangeInclusive, b"...", line, col)
            } else {
                Token::new(TokenKind::Range, b"..", line, col)
            }
        } else {
            Token::new(TokenKind::Dot, b".", line, col)
        }
    }

    fn match_question(&mut self, line: u32, col: u32) -> Token {
        if self.peek() == b'?' {
            self.advance();
            Token::new(TokenKind::NilCoalesce, b"??", line, col)
        } else {
            Token::new(TokenKind::Question, b"?", line, col)
        }
    }

    fn string_literal(&mut self, line: u32, col: u32) -> Token {
        let mut buf = [0u8; 256];
        let mut len = 0;
        
        while !self.is_at_end() && self.peek() != b'"' && len < 255 {
            if self.peek() == b'\\' {
                self.advance();
                if !self.is_at_end() {
                    let escaped = match self.advance() {
                        b'n' => b'
',
                        b't' => b'\t',
                        b'r' => b'\r',
                        b'\\' => b'\\',
                        b'"' => b'"',
                        b'0' => b'\0',
                        c => c,
                    };
                    buf[len] = escaped;
                    len += 1;
                }
            } else {
                buf[len] = self.advance();
                len += 1;
            }
        }
        
        if !self.is_at_end() {
            self.advance(); // closing "
        }
        
        Token::new(TokenKind::StringLiteral, &buf[..len], line, col)
    }

    fn number_literal(&mut self, first: u8, line: u32, col: u32) -> Token {
        let mut buf = [0u8; 64];
        let mut len = 0;
        buf[len] = first;
        len += 1;
        
        // Integerpartsplit
        while !self.is_at_end() && self.peek().is_ascii_digit() && len < 60 {
            buf[len] = self.advance();
            len += 1;
        }
        
        // smallnumberpartsplit
        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            buf[len] = self.advance(); // .
            len += 1;
            while !self.is_at_end() && self.peek().is_ascii_digit() && len < 60 {
                buf[len] = self.advance();
                len += 1;
            }
            Token::new(TokenKind::FloatLiteral, &buf[..len], line, col)
        } else {
            Token::new(TokenKind::IntegerLiteral, &buf[..len], line, col)
        }
    }

    fn identifier_or_keyword(&mut self, first: u8, line: u32, col: u32) -> Token {
        let mut buf = [0u8; 256];
        let mut len = 0;
        buf[len] = first;
        len += 1;
        
        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == b'_') && len < 255 {
            buf[len] = self.advance();
            len += 1;
        }
        
        let lexeme = &buf[..len];
        let kind = self.lookup_keyword(lexeme);
        Token::new(kind, lexeme, line, col)
    }

    fn lookup_keyword(&self, lexeme: &[u8]) -> TokenKind {
        match lexeme {
            b"func" => TokenKind::KwFunc,
            b"var" => TokenKind::KwVar,
            b"let" => TokenKind::KwLet,
            b"class" => TokenKind::KwClass,
            b"struct" => TokenKind::KwStruct,
            b"enum" => TokenKind::KwEnum,
            b"protocol" => TokenKind::KwProtocol,
            b"extension" => TokenKind::KwExtension,
            b"if" => TokenKind::KwIf,
            b"else" => TokenKind::KwElse,
            b"switch" => TokenKind::KwSwitch,
            b"case" => TokenKind::KwCase,
            b"default" => TokenKind::KwDefault,
            b"for" => TokenKind::KwFor,
            b"while" => TokenKind::KwWhile,
            b"do" => TokenKind::KwDo,
            b"break" => TokenKind::KwBreak,
            b"continue" => TokenKind::KwContinue,
            b"return" => TokenKind::KwReturn,
            b"throw" => TokenKind::KwThrow,
            b"try" => TokenKind::KwTry,
            b"catch" => TokenKind::KwCatch,
            b"finally" => TokenKind::KwFinally,
            b"import" => TokenKind::KwImport,
            b"export" => TokenKind::KwExport,
            b"public" => TokenKind::KwPublic,
            b"private" => TokenKind::KwPrivate,
            b"internal" => TokenKind::KwInternal,
            b"static" => TokenKind::KwStatic,
            b"mut" => TokenKind::KwMut,
            b"const" => TokenKind::KwConst,
            b"async" => TokenKind::KwAsync,
            b"await" => TokenKind::KwAwait,
            b"self" => TokenKind::KwSelf,
            b"super" => TokenKind::KwSuper,
            b"init" => TokenKind::KwInit,
            b"deinit" => TokenKind::KwDeinit,
            b"get" => TokenKind::KwGet,
            b"set" => TokenKind::KwSet,
            b"in" => TokenKind::KwIn,
            b"where" => TokenKind::KwWhere,
            b"guard" => TokenKind::KwGuard,
            b"defer" => TokenKind::KwDefer,
            b"true" | b"false" => TokenKind::BoolLiteral,
            b"nil" => TokenKind::NilLiteral,
            _ => TokenKind::Identifier,
        }
    }
}