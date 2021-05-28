//! strings you can subtract
//!
//! i promise this is something you need in your life, $100%
//!
//! ## example
//!
//! ```rust
//! use switchstring::Switchstring;
//!
//! let a = "I promise I love maths";
//! let b = "maths";
//! let c = "cute rustaceans such as ferris";
//! let d: Switchstring = "I promise ".into();
//! let improved = String::from(-d + a - b + c);
//! assert_eq!("I love cute rustaceans such as ferris", improved);
//! ```

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

/// a cow string that can be negative
#[derive(Clone, Debug, PartialEq, Eq)]
struct Inner<'a> {
    s: Cow<'a, str>,
    neg: Negation,
}

/// a string you can subtract from. you can add or subtract [`String`]s or [`&str`]s or other
/// [`Switchstring`]s from these.
///
/// to create one, use the `impl From<&str>`/`From<String>` implementations. to collapse one into a
/// normal [`String`], use the `impl From<Switchstring> for String`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Switchstring<'a> {
    this: Inner<'a>,
    // of course this is a linked list
    prev: Option<Arc<Switchstring<'a>>>,
}

impl<'a> Switchstring<'a> {
    /// prepends the given [`Switchstring`] to `self`, returning a new [`Switchstring`]
    fn prepend<'b: 'a>(&self, other: &Switchstring<'b>) -> Switchstring<'a> {
        // base case: self is the end of a list and other is an arbitrary substring
        // recursive case:
        if let Some(ref prev) = self.prev {
            Switchstring {
                this: self.this.clone(),
                prev: Some(Arc::new(prev.prepend(other))),
            }
        } else {
            // base case: directly put the next list in the next field
            Switchstring {
                this: self.this.clone(),
                prev: Some(Arc::new(other.clone())),
            }
        }
    }

    /// evaluates a [`Switchstring`] into an [`Inner`]. this allows for negative strings for
    /// absolutely deranged ideas such as adding a string to a negative string.
    fn eval(&self) -> Inner<'a> {
        // base case: end of the list
        // recursive case: evaluate rhs and then self
        if let Some(ref prev) = self.prev {
            let prefix = prev.eval();
            let suffix = &self.this;
            // println!("eval prefix {:?} suffix {:?}", suffix, prefix);
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
                // choice: -"a" + "b" = "b" (non negated will dominate, for practicality reasons)
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

// horrifying number of impls follow. i could not be bothered to write a macro,

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

impl<'a> Add<&str> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: &str) -> Self::Output {
        Switchstring::from(rhs).prepend(&self)
    }
}

impl<'a> Add<String> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: String) -> Self::Output {
        Switchstring::from(rhs).prepend(&self)
    }
}

impl<'a> Add<&str> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: &str) -> Self::Output {
        Switchstring::from(rhs).prepend(&self)
    }
}

impl<'a> Add<String> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: String) -> Self::Output {
        Switchstring::from(rhs).prepend(&self)
    }
}

impl<'a> Add<Switchstring<'a>> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: Switchstring<'a>) -> Self::Output {
        rhs.prepend(&self)
    }
}

impl<'a> Add<&Switchstring<'a>> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: &Switchstring<'a>) -> Self::Output {
        rhs.prepend(&self)
    }
}

impl<'a> Sub<Switchstring<'a>> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: Switchstring<'a>) -> Self::Output {
        self + -rhs
    }
}

impl<'a> Sub<&Switchstring<'a>> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: &Switchstring<'a>) -> Self::Output {
        self + -rhs
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

impl<'a> Add<Switchstring<'a>> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: Switchstring<'a>) -> Self::Output {
        rhs.prepend(&self)
    }
}

impl<'a> Add<&Switchstring<'a>> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn add(self, rhs: &Switchstring<'a>) -> Self::Output {
        rhs.prepend(&self)
    }
}

impl<'a> Sub<String> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: String) -> Self::Output {
        self + -Switchstring::from(rhs)
    }
}

impl<'a> Sub<&str> for Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: &str) -> Self::Output {
        self + -Switchstring::from(rhs)
    }
}

impl<'a> Sub<&str> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: &str) -> Self::Output {
        self + -Switchstring::from(rhs)
    }
}

impl<'a> Sub<Switchstring<'a>> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: Switchstring<'a>) -> Self::Output {
        self + -rhs
    }
}

impl<'a> Sub<&Switchstring<'a>> for &Switchstring<'a> {
    type Output = Switchstring<'a>;

    fn sub(self, rhs: &Switchstring<'a>) -> Self::Output {
        self + -rhs
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

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

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
        let b = "bb";
        let c = "cc";
        let s: String = (&a + b - c + c + &a - &a - b).into();
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

        let res = a.prepend(&b);

        assert_eq!(&exp, &res);

        let exp2 = Switchstring {
            this: Inner::new("cc".to_string(), false),
            prev: Some(Arc::new(exp)),
        };

        let c = Switchstring {
            this: Inner::new("cc".to_string(), false),
            prev: None,
        };

        let res = c.prepend(&res);

        assert_eq!(&exp2, &res);
    }
}
