use std::ops::{Add, Neg, Sub};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Negation {
    Negated,
    No,
}

impl Neg for Negation {
    type Output = Negation;

    fn neg(self) -> Self::Output {
        match self {
            Negation::No => Negation::Negated,
            Negation::Negated => Negation::No,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Inner {
    s: String,
    neg: Negation,
}

impl Inner {
    fn new(s: String, negated: bool) -> Inner {
        Inner {
            s,
            neg: match negated {
                true => Negation::Negated,
                false => Negation::No,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Substring {
    this: Inner,
    next: Option<Box<Substring>>,
}

impl Substring {
    fn concat(self, other: Substring) -> Substring {
        // base case: self is the end of a list and other is an arbitrary substring
        // recursive case:
        if let Some(next) = self.next {
            Substring {
                this: self.this,
                next: Some(Box::new(next.concat(other))),
            }
        } else {
            // base case: directly put the next list in the next field
            Substring {
                this: self.this,
                next: Some(Box::new(other)),
            }
        }
    }

    fn eval(self) -> Inner {
        // base case: end of the list
        // recursive case: evaluate rhs and then self
        if let Some(next) = self.next {
            let suffix = next.eval();
            let prefix = self.this;
            match (prefix.neg, suffix.neg) {
                (Negation::No, Negation::No) => Inner {
                    s: prefix.s + &suffix.s,
                    neg: Negation::No,
                },
                (Negation::No, Negation::Negated) => Inner {
                    s: prefix
                        .s
                        .strip_suffix(&suffix.s)
                        .map(|s| s.to_string())
                        .unwrap_or(prefix.s),
                    neg: Negation::No,
                },
                // choice: -"a" + "b" = "b" (non negated will dominate)
                (Negation::Negated, Negation::No) => Inner {
                    s: suffix
                        .s
                        .strip_prefix(&prefix.s)
                        .map(|s| s.to_string())
                        .unwrap_or(suffix.s),
                    neg: Negation::No,
                },
                (Negation::Negated, Negation::Negated) => Inner {
                    s: prefix.s + &suffix.s,
                    neg: Negation::Negated,
                },
            }
        } else {
            self.this
        }
    }
}

impl Neg for Substring {
    type Output = Substring;

    fn neg(self) -> Self::Output {
        Substring {
            this: Inner {
                neg: -self.this.neg,
                ..self.this
            },
            ..self
        }
    }
}

impl Add for Substring {
    type Output = Substring;

    fn add(self, rhs: Self) -> Self::Output {
        self.concat(rhs)
    }
}

impl Sub for Substring {
    type Output = Substring;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl From<Substring> for String {
    fn from(s: Substring) -> Self {
        match s.eval() {
            Inner {
                s,
                neg: Negation::No,
            } => s,
            Inner {
                s: _,
                neg: Negation::Negated,
            } => "".to_string(),
        }
    }
}

impl From<String> for Substring {
    fn from(s: String) -> Self {
        Substring {
            this: Inner {
                s,
                neg: Negation::No,
            },
            next: None,
        }
    }
}

impl From<&str> for Substring {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

#[cfg(test)]
mod tests {
    use super::{Inner, Substring};

    #[test]
    fn exprs() {
        // let a: Substring = "aa".into();
        // let b: Substring = "bb".into();
        // let c: Substring = "cc".into();
        // let s: String = (a + b - c + c + a - a).into();
    }

    #[test]
    fn concat() {
        let a = Substring {
            this: Inner::new("aa".to_string(), false),
            next: None,
        };

        let b = Substring {
            this: Inner::new("bb".to_string(), true),
            next: None,
        };

        let exp = Substring {
            this: Inner::new("aa".to_string(), false),
            next: Some(Box::new(Substring {
                this: Inner::new("bb".to_string(), true),
                next: None,
            })),
        };

        let res = a.concat(b);

        assert_eq!(&exp, &res);

        let exp2 = Substring {
            this: Inner::new("cc".to_string(), false),
            next: Some(Box::new(exp)),
        };

        let c = Substring {
            this: Inner::new("cc".to_string(), false),
            next: None,
        };

        let res = c.concat(res);

        assert_eq!(&exp2, &res);
    }
}
