use std::{collections::HashMap, fmt::Display, iter::Peekable, str::Chars};

use ordered_float::OrderedFloat;
use phf::phf_map;
use thiserror::Error;

use DoubleCharacterToken::*;
use Keyword::*;
use SingleCharacterToken::*;

/// The location of a [Token]"s lexeme in the
/// source code.
#[derive(Debug, Default, Clone)]
pub struct Location {
    /// Our vertical location in the file
    line_number: u16,

    /// Our horizontal location in the file
    column_number: u16,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Ln {}, Col {}", self.line_number, self.column_number))
    }
}

impl From<(u16, u16)> for Location {
    fn from((x, y): (u16, u16)) -> Self {
        Location {
            line_number: x,
            column_number: y,
        }
    }
}

impl Location {
    #[inline]
    fn advance_col(&mut self) {
        self.column_number += 1;
    }

    #[inline]
    fn advance_row(&mut self) {
        self.column_number = 0;
        self.line_number += 1;
    }
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected `{character}` at {location}")]
    UnexpectedCharacter { character: char, location: Location },
}

static LEXEME_TO_TOKEN_MAPPER: phf::Map<&'static str, Token> = phf_map! {
    // Map Keywords
    "and" => Token::KeywordToken(And),
    "class" => Token::KeywordToken(Class),
    "if" => Token::KeywordToken(If),
    "else" => Token::KeywordToken(Else),
    "true" => Token::KeywordToken(True),
    "false" => Token::KeywordToken(False),
    "fun" => Token::KeywordToken(Fun),
    "for" => Token::KeywordToken(For),
    "while" => Token::KeywordToken(While),
    "var" => Token::KeywordToken(Var),
    "nil" => Token::KeywordToken(Nil),
    "or" => Token::KeywordToken(Or),
    "print" => Token::KeywordToken(Print),
    "return" => Token::KeywordToken(Return),
    "super" => Token::KeywordToken(Super),
    "this" => Token::KeywordToken(This),

    // Map Single and Double character tokens
    "(" => Token::Single(LeftBrace),
    ")" => Token::Single(RightBrace),
    "{" => Token::Single(LeftParenthesis),
    "}" => Token::Single(RightParenthesis),
    "+" => Token::Single(Plus),
    "-" => Token::Single(Minus),
    "," => Token::Single(Comma),
    "." => Token::Single(Dot),
    ";" => Token::Single(SemiColon),
    "*" => Token::Single(Star),
    "!" => Token::Single(Not),
    "/" => Token::Single(Slash),
    "!=" => Token::Double(NotEqual),
    "=" => Token::Single(EqualSign),
    "==" => Token::Double(EqualEqualSign),
    "<" => Token::Single(LessThan),
    "<=" => Token::Double(LessThanOrEqual),
    ">" => Token::Single(GreaterThan),
    ">=" => Token::Double(GreaterThanOrEqual),
};

lazy_static! {
    static ref TOKEN_TO_LEXEME_MAPPER: HashMap<Token, &'static str> = {
        let mut mapper = HashMap::with_capacity(48);

        // Map Keywords
        mapper.insert(Token::KeywordToken(And), "and");
        mapper.insert(Token::KeywordToken(Class), "class");
        mapper.insert(Token::KeywordToken(If), "if");
        mapper.insert(Token::KeywordToken(Else), "else");
        mapper.insert(Token::KeywordToken(True), "true");
        mapper.insert(Token::KeywordToken(False), "false");
        mapper.insert(Token::KeywordToken(Fun), "fun");
        mapper.insert(Token::KeywordToken(For), "for");
        mapper.insert(Token::KeywordToken(While), "while");
        mapper.insert(Token::KeywordToken(Var), "var");
        mapper.insert(Token::KeywordToken(Nil), "nil");
        mapper.insert(Token::KeywordToken(Or), "or");
        mapper.insert(Token::KeywordToken(Print), "print");
        mapper.insert(Token::KeywordToken(Return), "return");
        mapper.insert(Token::KeywordToken(Super), "super");
        mapper.insert(Token::KeywordToken(This), "this");

        // Map Single and Double character tokens
        mapper.insert(Token::Single(LeftBrace), "(");
        mapper.insert(Token::Single(RightBrace), ")");
        mapper.insert(Token::Single(LeftParenthesis), "{");
        mapper.insert(Token::Single(RightParenthesis), "}");
        mapper.insert(Token::Single(Plus), "+");
        mapper.insert(Token::Single(Minus), "-");
        mapper.insert(Token::Single(Comma), ",");
        mapper.insert(Token::Single(Dot), ".");
        mapper.insert(Token::Single(SemiColon), ";");
        mapper.insert(Token::Single(Star), "*");
        mapper.insert(Token::Single(Not), "!");
        mapper.insert(Token::Single(Slash), "/");
        mapper.insert(Token::Double(NotEqual), "!=");
        mapper.insert(Token::Single(EqualSign), "=");
        mapper.insert(Token::Double(EqualEqualSign), "==");
        mapper.insert(Token::Single(LessThan), "<");
        mapper.insert(Token::Double(LessThanOrEqual), "<=");
        mapper.insert(Token::Single(GreaterThan), ">");
        mapper.insert(Token::Double(GreaterThanOrEqual), ">=");

        mapper
    };
}

/// All
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SingleCharacterToken {
    /// ## (
    LeftParenthesis,
    /// ## )
    RightParenthesis,
    /// ## {
    LeftBrace,
    /// ## }
    RightBrace,
    /// ## ,
    Comma,
    /// ## .
    Dot,
    /// ## -
    Minus,
    /// ## +
    Plus,
    /// ## ;
    SemiColon,
    /// ## /
    Slash,
    /// ## *
    Star,
    /// ## !
    Not,
    /// ## =
    EqualSign,
    /// ## >
    GreaterThan,
    /// ## <
    LessThan,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DoubleCharacterToken {
    /// ## !=
    NotEqual,
    /// ## ==
    EqualEqualSign,
    /// ## >=
    GreaterThanOrEqual,
    /// ## <=
    LessThanOrEqual,
}

/// Literals can be numbers, variable names, function names, class names, or strings
/// surrounded by double quotes `"`
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    Number(OrderedFloat<f32>),
    /// An identifier can be a variable name, a function name ...
    Identifier(String),
    /// A string is anything within double quotes `"<string>"`
    StringLiteral(String),
}

/// Keywords are literals that have been reserved for
/// the language"s internal use
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Keyword {
    /// ## and
    And,
    /// ## class
    Class,
    /// ## if
    If,
    /// ## else
    Else,
    /// ## true
    True,
    /// ## false
    False,
    /// ## fun
    Fun,
    /// ## for
    For,
    /// ## while
    While,
    /// ## var
    Var,
    /// ## nil
    Nil,
    /// ## or
    Or,
    /// ## print
    Print,
    /// ## return
    Return,
    /// ## super
    Super,
    /// ## this
    This,
}

/// All the valid tokens in the `lox` language
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Eof,
    LiteralToken(Literal),
    KeywordToken(Keyword),
    Single(SingleCharacterToken),
    Double(DoubleCharacterToken),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => f.write_str("END_OF_FILE"),
            Self::LiteralToken(literal) => match literal {
                Literal::Number(value) => f.write_str(&value.to_string()),
                Literal::Identifier(value) => f.write_str(value),
                Literal::StringLiteral(value) => f.write_str(value),
            },
            _ => f.write_str(TOKEN_TO_LEXEME_MAPPER.get(self).unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct TokenStream(Vec<Token>);

#[derive(Debug)]
pub struct Lexer {
    source_code: String,
    current_location: Location,
}

impl Lexer {
    /// Create a new lexer for the provided source code
    pub fn new(source_code: String) -> Self {
        Lexer {
            source_code,
            current_location: Location::default(),
        }
    }

    /// Scan the source code to generate a stream of [Token]`s producing
    /// [LexerError] if any errors are encountered.
    pub fn lex(&mut self) -> Result<TokenStream, Vec<LexerError>> {
        let mut errors = Vec::new();
        let mut tokens = Vec::with_capacity(self.source_code.len());
        let mut code = self.source_code.chars().peekable();
        while let Some(character) = code.next() {
            match character {
                ' ' | '\r' | '\t' => {}
                '\n' => self.current_location.advance_row(),
                '(' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get("(").cloned().unwrap()),
                ')' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get(")").cloned().unwrap()),
                '{' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get("{").cloned().unwrap()),
                '}' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get("}").cloned().unwrap()),
                '+' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get("+").cloned().unwrap()),
                '-' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get("-").cloned().unwrap()),
                ',' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get(",").cloned().unwrap()),
                '.' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get(".").cloned().unwrap()),
                ';' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get(";").cloned().unwrap()),
                '*' => tokens.push(LEXEME_TO_TOKEN_MAPPER.get("*").cloned().unwrap()),
                '!' => Self::add_double_or_single_token(&mut tokens, character, &mut code),
                '=' => Self::add_double_or_single_token(&mut tokens, character, &mut code),
                '<' => Self::add_double_or_single_token(&mut tokens, character, &mut code),
                '>' => Self::add_double_or_single_token(&mut tokens, character, &mut code),
                '/' => Self::consume_comment(&mut tokens, character, &mut self.current_location, &mut code),
                '"' => Self::add_string_literal(&mut tokens, &mut self.current_location, &mut code),
                '0'..='9' => {
                    Self::add_number_literal(&mut tokens, character, &mut self.current_location, &mut code)
                }
                'A'..='Z' | 'a'..='z' | '_' => Self::add_identifier_or_keyword(
                    &mut tokens,
                    character,
                    &mut self.current_location,
                    &mut code,
                ),
                _ => errors.push(LexerError::UnexpectedCharacter {
                    character,
                    location: self.current_location.clone(),
                }),
            }
            self.current_location.advance_col();
        }
        tokens.push(Token::Eof);
        if errors.is_empty() {
            Ok(TokenStream(tokens))
        } else {
            Err(errors)
        }
    }

    /// Look ahead one step. Add a [Token::Double] if the next character matched the expected
    /// character. Otherwise add a [Token::Single]
    fn add_double_or_single_token(tokens: &mut Vec<Token>, current_character: char, code: &mut Peekable<Chars>) {
        let expected_next_character = '=';
        if Self::one_step_look_ahead(expected_next_character, code) {
            // TODO: Advance column by 1
            let double_lexeme = format!("{}{}", current_character, expected_next_character);
            tokens.push(LEXEME_TO_TOKEN_MAPPER.get(&double_lexeme).cloned().unwrap());
        } else {
            let single_token = LEXEME_TO_TOKEN_MAPPER.get(&current_character.to_string());
            tokens.push(single_token.cloned().unwrap())
        }
    }

    /// Look ahead one character and if the next character is another '/`,
    /// consume the rest of the line. If not, add a single '/' token to the list
    /// to the list
    fn consume_comment(
        tokens: &mut Vec<Token>,
        current_character: char,
        current_location: &mut Location,
        code: &mut Peekable<Chars>,
    ) {
        let expected_next_char = '/';
        if Self::one_step_look_ahead(expected_next_char, code) {
            let mut advanced_iter = code.skip_while(|&character| character != '\n');
            match advanced_iter.next() {
                None => {}
                Some('\n') => current_location.advance_row(),
                Some(_) => panic!("we should never hit this arm"),
            }
        } else {
            let single_token = LEXEME_TO_TOKEN_MAPPER.get(&current_character.to_string());
            tokens.push(single_token.cloned().unwrap())
        }
    }

    /// Peek at the next character. If it is what we `expect`, we consume it by
    /// advancing the iterator then return `true`. Otherwise, we return false
    fn one_step_look_ahead(expect: char, code_characters: &mut Peekable<Chars>) -> bool {
        if let Some(next_character) = code_characters.peek() {
            match expect.cmp(next_character) {
                std::cmp::Ordering::Equal => {
                    code_characters.next();
                    return true;
                }
                _ => return false,
            }
        }
        // There is no next character. We are at the end of the file.
        false
    }

    /// Called when we encounter a `"`. we scan forward looking for
    /// a closing `"`. If we find one, we recognize the lexeme between
    /// the first `"` and the last  `"` we encountered as a string token.
    ///
    /// If a closing `"` is not found, that is we reach the end of the file before
    /// encountering another `"`, we record that as an error.
    fn add_string_literal(
        tokens: &mut Vec<Token>,
        current_location: &mut Location,
        code: &mut Peekable<Chars>,
    ) {
        let mut maybe_string = String::new();
        for character in code {
            if character == '"' {
                // We found the closing quotes of this string
                current_location.advance_col();
                tokens.push(Token::LiteralToken(Literal::StringLiteral(maybe_string)));
                return;
            } else {
                // We treat anything between the quotations as part of the string
                if character == '\n' {
                    current_location.advance_row();
                } else {
                    current_location.advance_col();
                }
                maybe_string.push(character);
            }
        }
        // If we consumed until the end but found not closing `"` we
        // emit an error.
        // TODO: Change the signature to take the list of errors
    }

    /// Called whenever we encounter a char digit.
    ///
    /// Consumes characters until we encounter a character that is neither
    /// a digit nor a `.` (decimal point)
    fn add_number_literal(
        tokens: &mut Vec<Token>,
        first_digit: char,
        current_location: &mut Location,
        code: &mut Peekable<Chars>,
    ) {
        let mut maybe_number = String::from(first_digit);
        while let Some(&character) = code.peek() {
            // Notice that unlike in the book, we allow users to write `123.`.
            // This will be interpreted as 123.0
            if character.is_ascii_digit() || character == '.' {
                maybe_number.push(character);
                code.next();
                current_location.advance_col();
            } else {
                // We've reached the end of the digit. We store a number token
                // TODO: What if the attempt to parse the number fails?
                // TODO: We should probably guard against numbers larger than f32::MAX
                let maybe_number_float: f32 = maybe_number.parse().unwrap();
                tokens.push(Token::LiteralToken(Literal::Number(OrderedFloat(maybe_number_float))));
                return;
            }
        }
    }

    /// Called whenever we encounter a character that is neither an operator
    /// nor part of a string literal. We interpret such as either parts
    /// of keywords or as variable identifiers.
    fn add_identifier_or_keyword(
        tokens: &mut Vec<Token>,
        first_character: char,
        current_location: &mut Location,
        code: &mut Peekable<Chars>,
    ) {
        let mut identifier_or_keyword = String::from(first_character);
        while let Some(&character) = code.peek() {
            match character {
                'A'..='Z' | 'a'..='z' | '_' | '0'..='9' => {
                    identifier_or_keyword.push(character);
                    code.next();
                    current_location.advance_col();
                }
                _ => {
                    // We've reached teh end of the keyword or identifier
                    match LEXEME_TO_TOKEN_MAPPER.get(&identifier_or_keyword) {
                        Some(keyword) => tokens.push(keyword.clone()),
                        None => tokens.push(Token::LiteralToken(Literal::Identifier(identifier_or_keyword))),
                    }
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_one_step_look_ahead() {
        todo!()
    }

    #[test]
    fn test_consume_comment() {
        todo!()
    }

    #[test]
    fn test_add_double_token() {
        todo!()
    }

    #[test]
    fn test_add_string_literal() {
        todo!()
    }

    #[test]
    fn test_add_number_literal() {
        todo!()
    }

    #[test]
    fn test_add_identifier_or_keyword() {
        todo!()
    }
}
