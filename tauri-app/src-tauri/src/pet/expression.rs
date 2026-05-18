use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct ExpressionContext {
    pub screen_w: i32,
    pub screen_h: i32,
    pub area_w: i32,
    pub area_h: i32,
    pub image_w: i32,
    pub image_h: i32,
    pub image_x: i32,
    pub image_y: i32,
    pub parent_x: i32,
    pub parent_y: i32,
    pub rand_s: i32,
    random_state: u64,
}

impl ExpressionContext {
    pub fn new(
        screen_w: i32,
        screen_h: i32,
        area_w: i32,
        area_h: i32,
        image_w: i32,
        image_h: i32,
    ) -> Self {
        Self {
            screen_w,
            screen_h,
            area_w,
            area_h,
            image_w,
            image_h,
            image_x: -1,
            image_y: -1,
            parent_x: -1,
            parent_y: -1,
            rand_s: 10,
            random_state: time_seed(),
        }
    }

    pub fn with_seed(mut self, random_seed: u64) -> Self {
        self.random_state = random_seed;
        self
    }

    pub fn with_spawn_random(mut self, rand_s: i32) -> Self {
        self.rand_s = rand_s;
        self
    }

    pub fn with_image_position(mut self, image_x: i32, image_y: i32) -> Self {
        self.image_x = image_x;
        self.image_y = image_y;
        self
    }

    pub fn with_parent_position(mut self, parent_x: i32, parent_y: i32) -> Self {
        self.parent_x = parent_x;
        self.parent_y = parent_y;
        self
    }

    fn next_random(&mut self) -> i32 {
        self.random_state = self
            .random_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        ((self.random_state >> 32) % 100) as i32
    }

    fn variable_value(&mut self, name: &str) -> Option<f64> {
        let value = match name {
            "screenW" => self.screen_w,
            "screenH" => self.screen_h,
            "areaW" => self.area_w,
            "areaH" => self.area_h,
            "imageW" => self.image_w,
            "imageH" => self.image_h,
            "imageX" => self.image_x,
            "imageY" => self.image_y,
            "parentX" => self.parent_x,
            "parentY" => self.parent_y,
            "randS" => self.rand_s,
            "random" => self.next_random(),
            _ => return None,
        };

        Some(value as f64)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionError {
    UnexpectedCharacter { character: char, position: usize },
    UnexpectedEnd,
    UnknownVariable(String),
    DivideByZero,
    TrailingInput { position: usize },
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter {
                character,
                position,
            } => write!(
                formatter,
                "unexpected character '{character}' at position {position}"
            ),
            Self::UnexpectedEnd => write!(formatter, "unexpected end of expression"),
            Self::UnknownVariable(name) => write!(formatter, "unknown variable '{name}'"),
            Self::DivideByZero => write!(formatter, "division by zero"),
            Self::TrailingInput { position } => {
                write!(formatter, "unexpected trailing input at position {position}")
            }
        }
    }
}

impl std::error::Error for ExpressionError {}

pub fn evaluate_expression(
    expression: &str,
    context: &mut ExpressionContext,
) -> Result<i32, ExpressionError> {
    let mut parser = Parser::new(expression, context);
    let value = parser.parse_expression()?;
    parser.skip_whitespace();

    if !parser.is_finished() {
        return Err(ExpressionError::TrailingInput {
            position: parser.position,
        });
    }

    Ok(value as i32)
}

struct Parser<'a, 'context> {
    input: &'a str,
    position: usize,
    context: &'context mut ExpressionContext,
}

impl<'a, 'context> Parser<'a, 'context> {
    fn new(input: &'a str, context: &'context mut ExpressionContext) -> Self {
        Self {
            input,
            position: 0,
            context,
        }
    }

    fn parse_expression(&mut self) -> Result<f64, ExpressionError> {
        let mut value = self.parse_term()?;

        loop {
            self.skip_whitespace();

            if self.consume_char('+') {
                value += self.parse_term()?;
            } else if self.consume_char('-') {
                value -= self.parse_term()?;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_term(&mut self) -> Result<f64, ExpressionError> {
        let mut value = self.parse_factor()?;

        loop {
            self.skip_whitespace();

            if self.consume_char('*') {
                value *= self.parse_factor()?;
            } else if self.consume_char('/') {
                let divisor = self.parse_factor()?;

                if divisor == 0.0 {
                    return Err(ExpressionError::DivideByZero);
                }

                value /= divisor;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<f64, ExpressionError> {
        self.skip_whitespace();

        if self.consume_char('+') {
            return self.parse_factor();
        }

        if self.consume_char('-') {
            return Ok(-self.parse_factor()?);
        }

        if self.consume_char('(') {
            let value = self.parse_expression()?;
            self.skip_whitespace();

            if !self.consume_char(')') {
                return Err(ExpressionError::UnexpectedEnd);
            }

            return Ok(value);
        }

        match self.peek_char() {
            Some(character) if character.is_ascii_digit() || character == '.' => self.parse_number(),
            Some(character) if character.is_ascii_alphabetic() => self.parse_variable(),
            Some(character) => Err(ExpressionError::UnexpectedCharacter {
                character,
                position: self.position,
            }),
            None => Err(ExpressionError::UnexpectedEnd),
        }
    }

    fn parse_number(&mut self) -> Result<f64, ExpressionError> {
        let start = self.position;
        let mut has_digit = false;
        let mut has_decimal = false;

        while let Some(character) = self.peek_char() {
            if character.is_ascii_digit() {
                has_digit = true;
                self.advance_char(character);
            } else if character == '.' && !has_decimal {
                has_decimal = true;
                self.advance_char(character);
            } else {
                break;
            }
        }

        if !has_digit {
            return Err(ExpressionError::UnexpectedCharacter {
                character: '.',
                position: start,
            });
        }

        self.input[start..self.position]
            .parse::<f64>()
            .map_err(|_| ExpressionError::UnexpectedCharacter {
                character: self.input[start..self.position]
                    .chars()
                    .next()
                    .unwrap_or('.'),
                position: start,
            })
    }

    fn parse_variable(&mut self) -> Result<f64, ExpressionError> {
        let start = self.position;

        while let Some(character) = self.peek_char() {
            if character.is_ascii_alphanumeric() || character == '_' {
                self.advance_char(character);
            } else {
                break;
            }
        }

        let name = &self.input[start..self.position];
        self.context
            .variable_value(name)
            .ok_or_else(|| ExpressionError::UnknownVariable(name.to_owned()))
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.peek_char() {
            if character.is_whitespace() {
                self.advance_char(character);
            } else {
                break;
            }
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.advance_char(expected);
            true
        } else {
            false
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance_char(&mut self, character: char) {
        self.position += character.len_utf8();
    }

    fn is_finished(&self) -> bool {
        self.position >= self.input.len()
    }
}

fn time_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(1)
}
