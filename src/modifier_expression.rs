//! Boolean expression evaluator for modifier conditions.
//!
//! This module provides parsing and evaluation of boolean expressions used to
//! determine whether a step should be executed based on package-level modifiers.
//!
//! # Syntax
//!
//! - Identifier: `nightly`, `experimental`, etc.
//! - Logical AND: `&`
//! - Logical OR: `|`
//! - Logical NOT: `!`
//! - Grouping: `(` and `)`
//!
//! # Examples
//!
//! - `nightly` - true if "nightly" modifier is defined
//! - `nightly & experimental` - true if both are defined
//! - `nightly | stable` - true if either is defined
//! - `!nightly` - true if "nightly" is NOT defined
//! - `(nightly | beta) & !windows` - complex expression with grouping

use anyhow::{Result, anyhow};

/// Evaluate a boolean expression against a set of defined modifiers.
///
/// # Arguments
///
/// * `expression` - The boolean expression string to evaluate
/// * `modifiers` - Set of modifier strings that are defined (evaluates to true)
///
/// # Returns
///
/// `true` if the expression evaluates to true, `false` otherwise
pub fn evaluate(expression: &str, modifiers: &[String]) -> Result<bool> {
    let tokens = tokenize(expression)?;
    let mut pos = 0;
    let result = parse_or(&tokens, &mut pos, modifiers)?;

    if pos < tokens.len() {
        return Err(anyhow!("Unexpected token at position {pos}"));
    }

    Ok(result)
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Identifier(String),
    And,
    Or,
    Not,
    LParen,
    RParen,
}

fn tokenize(expr: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {}
            '&' => tokens.push(Token::And),
            '|' => tokens.push(Token::Or),
            '!' => tokens.push(Token::Not),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' | '-' => {
                let mut ident = String::new();
                ident.push(ch);
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                        ident.push(ch);
                        let _ = chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Identifier(ident));
            }
            _ => {
                return Err(anyhow!("Unexpected character: '{ch}'"));
            }
        }
    }

    Ok(tokens)
}

// Parse OR expression (lowest precedence)
fn parse_or(tokens: &[Token], pos: &mut usize, modifiers: &[String]) -> Result<bool> {
    let mut result = parse_and(tokens, pos, modifiers)?;

    while *pos < tokens.len() && tokens[*pos] == Token::Or {
        *pos += 1;
        let right = parse_and(tokens, pos, modifiers)?;
        result = result || right;
    }

    Ok(result)
}

// Parse AND expression (higher precedence than OR)
fn parse_and(tokens: &[Token], pos: &mut usize, modifiers: &[String]) -> Result<bool> {
    let mut result = parse_not(tokens, pos, modifiers)?;

    while *pos < tokens.len() && tokens[*pos] == Token::And {
        *pos += 1;
        let right = parse_not(tokens, pos, modifiers)?;
        result = result && right;
    }

    Ok(result)
}

// Parse NOT expression (highest precedence for operators)
fn parse_not(tokens: &[Token], pos: &mut usize, modifiers: &[String]) -> Result<bool> {
    if *pos < tokens.len() && tokens[*pos] == Token::Not {
        *pos += 1;
        let result = parse_primary(tokens, pos, modifiers)?;
        Ok(!result)
    } else {
        parse_primary(tokens, pos, modifiers)
    }
}

// Parse primary expression (identifier or grouped expression)
fn parse_primary(tokens: &[Token], pos: &mut usize, modifiers: &[String]) -> Result<bool> {
    if *pos >= tokens.len() {
        return Err(anyhow!("Unexpected end of expression"));
    }

    match &tokens[*pos] {
        Token::Identifier(name) => {
            *pos += 1;
            Ok(modifiers.contains(name))
        }
        Token::LParen => {
            *pos += 1;
            let result = parse_or(tokens, pos, modifiers)?;
            if *pos >= tokens.len() || tokens[*pos] != Token::RParen {
                return Err(anyhow!("Missing closing parenthesis"));
            }
            *pos += 1;
            Ok(result)
        }
        _ => Err(anyhow!("Expected identifier or '(', got {:?}", tokens[*pos])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_identifier() {
        let modifiers = vec!["nightly".to_string()];
        assert!(evaluate("nightly", &modifiers).unwrap());
        assert!(!evaluate("stable", &modifiers).unwrap());
    }

    #[test]
    fn test_and_operator() {
        let modifiers = vec!["nightly".to_string(), "experimental".to_string()];
        assert!(evaluate("nightly & experimental", &modifiers).unwrap());
        assert!(!evaluate("nightly & stable", &modifiers).unwrap());
    }

    #[test]
    fn test_or_operator() {
        let modifiers = vec!["nightly".to_string()];
        assert!(evaluate("nightly | stable", &modifiers).unwrap());
        assert!(evaluate("stable | nightly", &modifiers).unwrap());
        assert!(!evaluate("stable | beta", &modifiers).unwrap());
    }

    #[test]
    fn test_not_operator() {
        let modifiers = vec!["nightly".to_string()];
        assert!(!evaluate("!nightly", &modifiers).unwrap());
        assert!(evaluate("!stable", &modifiers).unwrap());
    }

    #[test]
    fn test_parentheses() {
        let modifiers = vec!["nightly".to_string()];
        assert!(evaluate("(nightly)", &modifiers).unwrap());
        assert!(evaluate("(nightly | stable) & !beta", &modifiers).unwrap());
    }

    #[test]
    fn test_complex_expression() {
        let modifiers = vec!["nightly".to_string(), "experimental".to_string()];
        assert!(evaluate("(nightly | beta) & experimental", &modifiers).unwrap());
        assert!(!evaluate("(nightly | beta) & windows", &modifiers).unwrap());
        assert!(evaluate("nightly & (experimental | stable)", &modifiers).unwrap());
    }

    #[test]
    fn test_operator_precedence() {
        let modifiers = vec!["a".to_string(), "b".to_string()];
        // a | b & c should be parsed as a | (b & c)
        assert!(evaluate("a | b & c", &modifiers).unwrap()); // true because a is true
        assert!(evaluate("d | b & a", &modifiers).unwrap()); // true because b & a is true
        assert!(!evaluate("d | e & a", &modifiers).unwrap()); // false because both sides are false
    }

    #[test]
    fn test_not_precedence() {
        let modifiers = vec!["a".to_string()];
        assert!(!evaluate("!a & b", &modifiers).unwrap()); // (!a) & b = false & false = false
        assert!(evaluate("!b & a", &modifiers).unwrap()); // (!b) & a = true & true = true
    }

    #[test]
    fn test_empty_modifiers() {
        let modifiers = vec![];
        assert!(!evaluate("nightly", &modifiers).unwrap());
        assert!(evaluate("!nightly", &modifiers).unwrap());
    }

    #[test]
    fn test_whitespace() {
        let modifiers = vec!["nightly".to_string()];
        assert!(evaluate("  nightly  ", &modifiers).unwrap());
        assert!(evaluate("nightly|stable", &modifiers).unwrap());
        assert!(evaluate("  ( nightly | stable )  ", &modifiers).unwrap());
    }

    #[test]
    fn test_invalid_expressions() {
        let modifiers = vec![];
        assert!(evaluate("", &modifiers).is_err());
        assert!(evaluate("(nightly", &modifiers).is_err());
        assert!(evaluate("nightly)", &modifiers).is_err());
        assert!(evaluate("&", &modifiers).is_err());
        assert!(evaluate("nightly &", &modifiers).is_err());
    }
}
