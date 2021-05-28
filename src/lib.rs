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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Substring<'a> {
    this: Inner<'a>,
    next: Option<Arc<Substring<'a>>>,
}

impl<'a> Substring<'a> {
    fn concat<'b: 'a>(&self, other: &Substring<'b>) -> Substring<'a> {
        // base case: self is the end of a list and other is an arbitrary substring
        // recursive case:
        if let Some(ref next) = self.next {
            Substring {
                this: self.this.clone(),
                next: Some(Arc::new(next.concat(other))),
            }
        } else {
            // base case: directly put the next list in the next field
            Substring {
                this: self.this.clone(),
                next: Some(Arc::new(other.clone())),
            }
        }
    }

    fn eval(&self) -> Inner<'a> {
        // base case: end of the list
        // recursive case: evaluate rhs and then self
        if let Some(ref next) = self.next {
            let suffix = next.eval();
            let prefix = &self.this;
            println!("eval prefix {:?} suffix {:?}", suffix, prefix);
            match (prefix.neg, suffix.neg) {
                (Negation::No, Negation::No) => Inner {
                    s: prefix.s.clone() + suffix.s,
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
                        .unwrap_or(suffix.s),
                    neg: Negation::No,
                },
                (Negation::Negated, Negation::Negated) => Inner {
                    s: prefix.s.clone() + suffix.s,
                    neg: Negation::Negated,
                },
            }
        } else {
            self.this.clone()
        }
    }
}

impl<'a> AsRef<Substring<'a>> for Substring<'a> {
    fn as_ref(&self) -> &Substring<'a> {
        self
    }
}

impl<'a> Neg for Substring<'a> {
    type Output = Substring<'a>;

    fn neg(self) -> Self::Output {
        Substring {
            this: Inner {
                s: self.this.s.clone(),
                neg: -self.this.neg,
            },
            next: self.next.clone(),
        }
    }
}

impl<'a, AR: AsRef<Substring<'a>>> Add<AR> for Substring<'a> {
    type Output = Substring<'a>;

    fn add(self, rhs: AR) -> Self::Output {
        self.concat(rhs.as_ref())
    }
}

impl<'a, AR: AsRef<Substring<'a>>> Sub<AR> for Substring<'a> {
    type Output = Substring<'a>;

    fn sub(self, rhs: AR) -> Self::Output {
        self + -(rhs.as_ref())
    }
}

impl<'a> Neg for &Substring<'a> {
    type Output = Substring<'a>;

    fn neg(self) -> Self::Output {
        Substring {
            this: Inner {
                s: self.this.s.clone(),
                neg: -self.this.neg,
            },
            next: self.next.clone(),
        }
    }
}

impl<'a, AR: AsRef<Substring<'a>>> Add<AR> for &Substring<'a> {
    type Output = Substring<'a>;

    fn add(self, rhs: AR) -> Self::Output {
        self.concat(rhs.as_ref())
    }
}

impl<'a, AR: AsRef<Substring<'a>>> Sub<AR> for &Substring<'a> {
    type Output = Substring<'a>;

    fn sub(self, rhs: AR) -> Self::Output {
        self + -rhs.as_ref()
    }
}

impl<'a> From<Substring<'a>> for String {
    fn from(s: Substring) -> Self {
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

impl<'a> From<String> for Substring<'a> {
    fn from(s: String) -> Self {
        Substring {
            this: Inner {
                s: Cow::Owned(s),
                neg: Negation::No,
            },
            next: None,
        }
    }
}

impl<'a> From<&str> for Substring<'a> {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{Inner, Substring};

    #[test]
    fn exprs() {
        let a: Substring = "aa".into();
        let b: Substring = "bb".into();
        let c: Substring = "cc".into();
        let s: String = (&a + &b - &c + &c + &a - &a).into();
        assert_eq!("aabb", &s);
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
            next: Some(Arc::new(Substring {
                this: Inner::new("bb".to_string(), true),
                next: None,
            })),
        };

        let res = a.concat(&b);

        assert_eq!(&exp, &res);

        let exp2 = Substring {
            this: Inner::new("cc".to_string(), false),
            next: Some(Arc::new(exp)),
        };

        let c = Substring {
            this: Inner::new("cc".to_string(), false),
            next: None,
        };

        let res = c.concat(&res);

        assert_eq!(&exp2, &res);
    }
}
