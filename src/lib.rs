use std::collections::HashMap;

// Value API

#[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Box<Value>>),
    Obj(HashMap<String, Box<Value>>),
}

// Serialization API

pub fn serialize_value(value: &Value) -> String {
    match value {
        Value::Null => String::from("null"),
        Value::Bool(bool_value) => bool_value.to_string(),
        Value::Num(num_value) => num_value.to_string(),
        Value::Str(str_value) => format!("\"{}\"", str_value),
        Value::Arr(arr_value) => format!(
            "[{}]",
            arr_value
                .iter()
                .map(|boxed_value| serialize_value(&boxed_value))
                .fold(String::new(), |mut concat_str, value_str| {
                    if !concat_str.is_empty() {
                        concat_str.push_str(",");
                    }

                    concat_str.push_str(&value_str);
                    concat_str
                })
        ),
        Value::Obj(obj_value) => format!(
            "{{{}}}",
            obj_value
                .iter()
                .map(|boxed_value| (boxed_value.0, serialize_value(&boxed_value.1)))
                .fold(String::new(), |mut concat_str, value_tuple| {
                    if !concat_str.is_empty() {
                        concat_str.push_str(",");
                    }

                    concat_str.push_str(&format!("\"{}\"", value_tuple.0));
                    concat_str.push_str(": ");
                    concat_str.push_str(&value_tuple.1);
                    concat_str
                })
        ),
    }
}

// Deserialization API

#[derive(Debug, PartialEq)]
enum TokenKind {
    // Single-character tokens.
    LeftSquaredBrace,
    RightSquaredBrace,
    LeftCurlyBrace,
    RightCurlyBrace,
    Comma,
    Colon,

    // Literals
    Str(String),
    Num(String),
    True,
    False,
    Null,

    // Special
    Eof,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    line: usize,
}

struct Scanner<'a> {
    buffer: &'a [u8],
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    errors: Vec<String>,
}

impl<'a> Scanner<'a> {
    fn new(buffer: &'a str) -> Scanner {
        Scanner {
            buffer: buffer.as_bytes(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            errors: Vec::new(),
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.add_token(TokenKind::Eof);
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            // skip whitespaces.
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            // Single-character tokens.
            '[' => self.add_token(TokenKind::LeftSquaredBrace),
            ']' => self.add_token(TokenKind::RightSquaredBrace),
            '{' => self.add_token(TokenKind::LeftCurlyBrace),
            '}' => self.add_token(TokenKind::RightCurlyBrace),
            ',' => self.add_token(TokenKind::Comma),
            ':' => self.add_token(TokenKind::Colon),

            // String literal.
            '"' => self.add_string_token(),

            // Number literal.
            '-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                self.add_number_token(c);
            }

            // Keyword literals.
            't' => {
                if !self.try_match_str("rue") {
                    self.report_error("Unexpected token!");
                }
                self.add_token(TokenKind::True);
            }

            'f' => {
                if !self.try_match_str("alse") {
                    self.report_error("Unexpected token!");
                }
                self.add_token(TokenKind::False);
            }

            'n' => {
                if !self.try_match_str("ull") {
                    self.report_error("Unexpected token!");
                }
                self.add_token(TokenKind::Null);
            }

            _ => {
                self.report_error("Unexpected character!");
            }
        }
    }

    fn add_token(&mut self, token_kind: TokenKind) {
        self.tokens.push(Token {
            kind: token_kind,
            line: self.line,
        });
    }

    fn add_string_token(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                '\n' => self.line += 1,
                '"' => break,
                _ => (),
            }

            self.advance();
        }

        if self.is_at_end() {
            self.report_error("Unterminated string!");
            return;
        }

        self.advance();
        self.add_token(TokenKind::Str(
            String::from_utf8(self.buffer[self.start + 1..self.current - 1].to_vec()).unwrap(),
        ));
    }

    fn add_number_token(&mut self, first_c: char) {
        // Parse the integer part.
        let mut num_integer_digits = if first_c == '-' { 0 } else { 1 };
        let mut is_first_integer_digit_0 = first_c == '0';

        while self.peek().is_ascii_digit() {
            let c = self.advance();

            num_integer_digits += 1;
            if c == '0' {
                if num_integer_digits == 1 {
                    is_first_integer_digit_0 = true;
                } else if num_integer_digits == 2 && is_first_integer_digit_0 {
                    self.report_error("Invalid number literal!");
                }
            }
        }

        // Parse the fraction part.
        if self.peek() == '.' {
            self.advance();

            let mut has_fraction_digit = false;
            while self.peek().is_ascii_digit() {
                has_fraction_digit = true;
                self.advance();
            }

            if !has_fraction_digit {
                self.report_error("Invalid number literal!");
            }
        }

        // Parse the exponent part.
        match self.peek() {
            'e' | 'E' => {
                self.advance();
                match self.peek() {
                    '+' | '-' => {
                        self.advance();
                        ()
                    }
                    _ => (),
                }

                let mut has_exponent_digit = false;
                while self.peek().is_ascii_digit() {
                    has_exponent_digit = true;
                    self.advance();
                }

                if !has_exponent_digit {
                    self.report_error("Invalid number literal!");
                }
            }
            _ => (),
        }

        self.add_token(TokenKind::Num(
            String::from_utf8(self.buffer[self.start..self.current].to_vec()).unwrap(),
        ));
    }

    fn try_match_str(&mut self, expected: &str) -> bool {
        return expected.chars().all(|c| self.try_match(c));
    }

    fn try_match(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }

        self.advance();
        return true;
    }

    fn advance(&mut self) -> char {
        let result = self.peek();
        self.current += 1;

        result
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.buffer[self.current] as char
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.buffer.len();
    }

    fn report_error(&mut self, message: &str) {
        self.errors
            .push(format!("[Line {}] Error: {}", self.line, message));
    }
}

struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    fn parse_tokens(&mut self) -> Option<Value> {
        let maybe_value = self.parse_value();

        if self.is_at_end() || self.peek().kind != TokenKind::Eof {
            self.report_error("Expected EOF!");
            return None;
        }

        return maybe_value;
    }

    fn parse_value(&mut self) -> Option<Value> {
        if self.is_at_end() {
            self.report_error("Unexpected EOF!");
            return None;
        }

        let first_token = self.peek();
        match &first_token.kind {
            TokenKind::Str(str_value) => {
                self.advance();
                return Some(Value::Str(str_value.clone()));
            }

            TokenKind::Num(str_value) => {
                self.advance();
                return Some(Value::Num(str_value.parse::<f64>().unwrap()));
            }

            TokenKind::True => {
                self.advance();
                return Some(Value::Bool(true));
            }

            TokenKind::False => {
                self.advance();
                return Some(Value::Bool(false));
            }

            TokenKind::Null => {
                self.advance();
                return Some(Value::Null);
            }

            TokenKind::LeftCurlyBrace => {
                return self.parse_object();
            }

            TokenKind::LeftSquaredBrace => {
                return self.parse_array();
            }

            _ => {
                self.report_error_with_token(first_token, "Unexpected token!");
                return None;
            }
        }
    }

    fn parse_object(&mut self) -> Option<Value> {
        if self.is_at_end() {
            self.report_error("Unexpected EOF!");
            return None;
        }

        if !self.try_match(TokenKind::LeftCurlyBrace) {
            self.report_error_with_token(self.peek(), "Expected left curly brace!");
            return None;
        }

        let mut result_obj = HashMap::<String, Box<Value>>::new();
        let mut previous_match_is_member = false;
        loop {
            if self.is_at_end() {
                self.report_error("Unexpected EOF!");
                return None;
            }

            if self.try_match(TokenKind::RightCurlyBrace) {
                return Some(Value::Obj(result_obj));
            }

            let next_token = self.peek();
            if previous_match_is_member {
                if !self.try_match(TokenKind::Comma) {
                    self.report_error_with_token(
                        next_token,
                        "JSON object members must be separated by a comma!",
                    );
                    return None;
                }
            }

            let maybe_member = self.parse_object_member();
            if maybe_member.is_none() {
                self.report_error_with_token(next_token, "Failed to parse JSON object member!");
                return None;
            }

            let member = maybe_member.unwrap();
            if result_obj.contains_key(&member.0) {
                self.report_error_with_token(
                    next_token,
                    &format!("Duplicate JSON object key! Key {}", &member.0),
                );
                return None;
            }

            result_obj.insert(member.0, Box::new(member.1));
            previous_match_is_member = true;
        }
    }

    fn parse_object_member(&mut self) -> Option<(String, Value)> {
        if self.is_at_end() {
            self.report_error("Unexpected EOF!");
            return None;
        }

        let result_key;
        let result_value;

        let next_token = self.peek();
        match &next_token.kind {
            TokenKind::Str(str_value) => {
                result_key = str_value.clone();
                self.advance();
            }

            _ => {
                self.report_error_with_token(next_token, "Expected JSON object element key!");
                return None;
            }
        }

        if self.is_at_end() {
            self.report_error("Unexpected EOF!");
            return None;
        }

        if !self.try_match(TokenKind::Colon) {
            self.report_error_with_token(
                self.peek(),
                "Expected JSON object element key-value colon separator!",
            );
        }

        result_value = self.parse_value();
        if result_value.is_none() {
            self.report_error("Failed to parse JSON object element value!");
            return None;
        }

        Some((result_key, result_value.unwrap()))
    }

    fn parse_array(&mut self) -> Option<Value> {
        if self.is_at_end() {
            self.report_error("Unexpected EOF!");
            return None;
        }

        if !self.try_match(TokenKind::LeftSquaredBrace) {
            self.report_error_with_token(self.peek(), "Expected left squared brace!");
            return None;
        }

        let mut result_arr = Vec::<Box<Value>>::new();
        let mut previous_match_is_element = false;
        loop {
            if self.is_at_end() {
                self.report_error("Unexpected EOF!");
                return None;
            }

            if self.try_match(TokenKind::RightSquaredBrace) {
                return Some(Value::Arr(result_arr));
            }

            let next_token = self.peek();
            if previous_match_is_element {
                if !self.try_match(TokenKind::Comma) {
                    self.report_error_with_token(
                        next_token,
                        "JSON array elements must be separated by a comma!",
                    );
                    return None;
                }
            }

            let maybe_element = self.parse_value();
            if maybe_element.is_none() {
                self.report_error_with_token(next_token, "Failed to parse JSON object element!");
                return None;
            }

            let element = maybe_element.unwrap();
            result_arr.push(Box::new(element));
            previous_match_is_element = true;
        }
    }

    fn try_match(&mut self, expected: TokenKind) -> bool {
        if self.is_at_end() || self.peek().kind != expected {
            return false;
        }

        self.advance();
        return true;
    }

    fn advance(&mut self) -> &'a Token {
        let result = self.peek();
        self.current += 1;

        result
    }

    fn peek(&self) -> &'a Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.tokens.len();
    }

    fn report_error_with_token(&mut self, token: &Token, message: &str) {
        self.errors
            .push(format!("[Line {}] Error: {}", token.line, message));
    }

    fn report_error(&mut self, message: &str) {
        self.errors.push(format!("Error: {}", message));
    }
}

pub fn deserialize_value(buffer: &str) -> Option<Value> {
    let mut scanner = Scanner::new(buffer);
    scanner.scan_tokens();

    // println!("Scanner tokens = {:?}", scanner.tokens);
    // println!("Scanner errors = {:?}", scanner.errors);
    if !scanner.errors.is_empty() {
        println!("Scanner failed with errors!");
        return None;
    }

    let mut parser = Parser::new(&scanner.tokens);
    let maybe_value = parser.parse_tokens();

    // println!("Parser result = {:?}", maybe_value);
    // println!("Parser errors = {:?}", parser.errors);
    if !parser.errors.is_empty() {
        println!("Parser failed with errors!");
        return None;
    }

    maybe_value
}
