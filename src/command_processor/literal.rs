#[derive(PartialEq, Clone, Debug)]
pub enum Literal {
    StringLiteral(String),
    IntegerLiteral(i64),
    BooleanLiteral(bool),
}

impl Literal {
    pub fn integer_from_string(input: String) -> Option<Literal> {
        return input.parse::<i64>().ok().map(|i| Literal::from(i));
    }

    pub fn bool_from_string(input: String) -> Option<Literal> {
        if input.to_lowercase() == "true" {
            return Some(Literal::from(true));
        } else if input.to_lowercase() == "false" {
            return Some(Literal::from(false));
        } else {
            return None;
        }
    }

    pub fn is_string(&self) -> bool {
        return std::mem::discriminant(self) == std::mem::discriminant(&Literal::from(""));
    }

    pub fn is_integer(&self) -> bool {
        return std::mem::discriminant(self) == std::mem::discriminant(&Literal::from(0));
    }

    pub fn is_bool(&self) -> bool {
        return std::mem::discriminant(self) == std::mem::discriminant(&Literal::from(true));
    }

    pub fn integer_value(&self) -> Option<i64> {
        return match self {
            Literal::StringLiteral(_) => None,
            Literal::IntegerLiteral(i) => Some(*i),
            Literal::BooleanLiteral(_) => None,
        };
    }

    pub fn string_value(&self) -> String {
        return match self {
            Literal::StringLiteral(s) => s.clone(),
            Literal::IntegerLiteral(i) => format!("{}", i),
            Literal::BooleanLiteral(b) => {
                if *b {
                    String::from("True")
                } else {
                    String::from("False")
                }
            }
        };
    }
}

impl From<&str> for Literal {
    fn from(s: &str) -> Self {
        return Self::StringLiteral(s.to_string());
    }
}

impl From<String> for Literal {
    fn from(s: String) -> Self {
        return Self::StringLiteral(s);
    }
}

impl From<i64> for Literal {
    fn from(i: i64) -> Self {
        return Self::IntegerLiteral(i);
    }
}

impl From<bool> for Literal {
    fn from(b: bool) -> Self {
        return Self::BooleanLiteral(b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_string() {
        let string_literal = Literal::from("test");
        let integer_literal = Literal::from(123);
        let boolean_literal = Literal::from(true);

        assert!(string_literal.is_string());
        assert!(!integer_literal.is_string());
        assert!(!boolean_literal.is_string());
    }

    #[test]
    fn test_is_integer() {
        let string_literal = Literal::from("test");
        let integer_literal = Literal::from(123);
        let boolean_literal = Literal::from(true);

        assert!(!string_literal.is_integer());
        assert!(integer_literal.is_integer());
        assert!(!boolean_literal.is_integer());
    }

    #[test]
    fn test_is_bool() {
        let string_literal = Literal::from("test");
        let integer_literal = Literal::from(123);
        let boolean_literal = Literal::from(true);

        assert!(!string_literal.is_bool());
        assert!(!integer_literal.is_bool());
        assert!(boolean_literal.is_bool());
    }
}
