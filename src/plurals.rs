use std::fmt;

use self::Resolver::Function;

pub enum Resolver {
    Function(Box<Fn(u64) -> usize>),
}

impl Resolver {
    /// Resolver returns the number of the correct plural form
    /// for `n` objects, as defined by the rule contained in this resolver.
    pub fn resolve(&self, n: u64) -> usize {
        match *self {
            Function(ref func) => func(n),
        }
    }
}

impl fmt::Debug for Resolver {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Function(..) => fmt.write_str("Function(..)"),
        }
    }
}
