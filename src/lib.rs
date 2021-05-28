use std::{
    borrow::Cow,
    ops::{Add, Neg, Sub},
    sync::Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
struct Inner<'a> {
    s: Cow<'a, str>,
    neg: Negation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Switchstring<'a> {
    this: Inner<'a>,
    prev: Option<Arc<Switchstring<'a>>>,
}

impl<'a> Switchstring<'a> {
    fn concat<'b: 'a>(&self, other: &Switchstring<'b>) -> Switchstring<'a> {
        // base case: self is the end of a list and other is an arbitrary substring
        // recursive case:
        if let Some(ref next) = self.prev {
            Switchstring {
                this: self.this.clone(),
                prev: Some(Arc::new(next.concat(other))),
            }
        } else {
            // base case: directly put the next list in the next field
            Switchstring {
                this: self.this.clone(),
                prev: Some(Arc::new(other.clone())),
            }
        }
    }

    fn eval(&self) -> Inner<'a> {
        // base case: end of the list
        // recursive case: evaluate rhs and then self
        if let Some(ref prev) = self.prev {
            let prefix = prev.eval();
            let suffix = &self.this;
            println!("eval prefix {:?} suffix {:?}", suffix, prefix);
            match (prefix.neg, suffix.neg) {
                (Negation::No, Negation::No) => Inner {
                    s: prefix.s + suffix.s.clone(),
                    neg: Negation::No,
                },
                (Negation::No, Negation::Negated) => Inner {
                    s: prefix
                        .s
                        .strip_suffix(suffix.s.as_ref())
                        .map(|s| Cow::Owned(s.to_string()))
                        .unwrap_or_else(|| prefix.s.to_owned()),
                    neg: Negation::No,
                },
                // choice: -"a" + "b" = "b" (non negated will dominate)
                (Negation::Negated, Negation::No) => Inner {
                    s: suffix
                        .s
                        .strip_prefix(prefix.s.as_ref())
                        .map(|s| Cow::Owned(s.to_string()))
                        .unwrap_or_else(|| suffix.s.clone()),
                    neg: Negation::No,
                },
                (Negation::Negated, Negation::Negated) => Inner {
                    s: prefix.s + suffix.s.clone(),
                    neg: Negation::Negated,
                },
            }
        } else {
            self.this.clone()
        }
    }
}

impl<'a> AsRef<Switchstring<'a>> for Switchstring<'a> {
    fn as_ref(&self) -> &Switchstring<'a> {
        self
    }
}

impl<'a> Neg for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn neg(self) -> Self::Output {
        Switchstring {
            this: Inner {
                s: self.this.s.clone(),
                neg: -self.this.neg,
            },
            prev: self.prev.clone(),
        }
    }
}

impl<'a, AR: AsRef<Switchstring<'a>>> Add<AR> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: AR) -> Self::Output {
        rhs.as_ref().concat(&self)
    }
}

impl<'a, AR: AsRef<Switchstring<'a>>> Sub<AR> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: AR) -> Self::Output {
        self + -(rhs.as_ref())
    }
}

impl<'a> Neg for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn neg(self) -> Self::Output {
        Switchstring {
            this: Inner {
                s: self.this.s.clone(),
                neg: -self.this.neg,
            },
            prev: self.prev.clone(),
        }
    }
}

impl<'a, AR: AsRef<Switchstring<'a>>> Add<AR> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: AR) -> Self::Output {
        rhs.as_ref().concat(&self)
    }
}

impl<'a, AR: AsRef<Switchstring<'a>>> Sub<AR> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: AR) -> Self::Output {
        self + -rhs.as_ref()
    }
}

impl<'a> From<Switchstring<'a>> for String {
    fn from(s: Switchstring) -> Self {
        match s.eval() {
            Inner {
                s,
                neg: Negation::No,
            } => s.to_string(),
            Inner {
                s: _,
                neg: Negation::Negated,
            } => "".to_string(),
        }
    }
}

impl<'a> From<String> for Switchstring<'a> {
    fn from(s: String) -> Self {
        Switchstring {
            this: Inner {
                s: Cow::Owned(s),
                neg: Negation::No,
            },
            prev: None,
        }
    }
}

impl<'a> From<&str> for Switchstring<'a> {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, sync::Arc};

    use crate::Negation;

    use super::{Inner, Switchstring};

    impl<'a> Inner<'a> {
        fn new(s: String, negated: bool) -> Inner<'a> {
            Inner {
                s: Cow::Owned(s),
                neg: match negated {
                    true => Negation::Negated,
                    false => Negation::No,
                },
            }
        }
    }

    #[test]
    fn exprs() {
        let a: Switchstring = "aa".into();
        let b: Switchstring = "bb".into();
        let c: Switchstring = "cc".into();
        let s: String = (&a + &b - &c + &c + &a - &a - &b).into();
        assert_eq!("aabbcc", &s);
    }

    #[test]
    fn concat() {
        let a = Switchstring {
            this: Inner::new("aa".to_string(), false),
            prev: None,
        };

        let b = Switchstring {
            this: Inner::new("bb".to_string(), true),
            prev: None,
        };

        let exp = Switchstring {
            this: Inner::new("aa".to_string(), false),
            prev: Some(Arc::new(Switchstring {
                this: Inner::new("bb".to_string(), true),
                prev: None,
            })),
        };

        let res = a.concat(&b);

        assert_eq!(&exp, &res);

        let exp2 = Switchstring {
            this: Inner::new("cc".to_string(), false),
            prev: Some(Arc::new(exp)),
        };

        let c = Switchstring {
            this: Inner::new("cc".to_string(), false),
            prev: None,
        };

        let res = c.concat(&res);

        assert_eq!(&exp2, &res);
    }
}
