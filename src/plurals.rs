use std::fmt;

use self::Resolver::*;
pub enum Resolver<'a> {
    /// A function/closure manually supplied by the user.
    Function(Box<Fn(u64) -> usize>),
    /// A boolean expression
    Expr(&'a str, Option<Ast<'a>>),
}

use self::Ast::*;
#[derive(Debug)]
enum Ast<'a> {
    /// A ternary expression
    /// x ? a : b
    ///
    /// the three Ast<'a> are respectively x, a and b.
    Ternary(&'a Ast<'a>, &'a Ast<'a>, &'a Ast<'a>),
    /// The n variable.
    N,
    /// Integer literals.
    Integer(u64),
    /// Boolean literals.
    Bool(bool),
    /// Comparison operators.
    CompOp(&'a str, &'a Ast<'a>, &'a Ast<'a>),
    /// && or || operators.
    CombOp(&'a str, &'a Ast<'a>, &'a Ast<'a>),
    /// ! operator.
    Not(&'a Ast<'a>),
}

impl<'a> Ast<'a> {
    fn resolve(&self, n: u64) -> usize {
        match *self {
            Ternary(cond, ok, nok) => if cond.resolve(n) == 0 {
                nok.resolve(n)
            } else {
                ok.resolve(n)
            },
            N => n as usize,
            Integer(x) => x as usize,
            Bool(b) => b as usize,
            CompOp(op, lhs, rhs) => (match op {
                "==" => lhs.resolve(n) == rhs.resolve(n),
                "!=" => lhs.resolve(n) != rhs.resolve(n),
                ">=" => lhs.resolve(n) >= rhs.resolve(n),
                "<=" => lhs.resolve(n) <= rhs.resolve(n),
                ">" => lhs.resolve(n) > rhs.resolve(n),
                "<" => lhs.resolve(n) < rhs.resolve(n),
                _ => unreachable!(),
            }) as usize,
            CombOp(op, lhs, rhs) => (match op {
                "&&" => lhs.resolve(n) != 0 && rhs.resolve(n) != 0,
                "||" => lhs.resolve(n) != 0 || rhs.resolve(n) != 0,
                _ => unreachable!()
            }) as usize,
            Not(val) => match val.resolve(n) {
                0 => 1,
                _ => 0,
            }
        }
    }
}

impl<'a> Resolver<'a> {
    /// Returns the number of the correct plural form
    /// for `n` objects, as defined by the rule contained in this resolver.
    pub fn resolve(&self, n: u64) -> usize {
        match *self {
            Function(ref func) => func(n),
            Expr(expr, ref ast) => {
                if let Some(ast) = ast {
                    ast.resolve(n)
                } else {
                    // TODO: parse expr
                    unimplemented!()
                }
            },
        }
    }
}

impl<'a> fmt::Debug for Resolver<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Function(..) => fmt.write_str("Function(..)"),
            Expr(expr, ref ast) => fmt.write_fmt(format_args!("Expr({}, {:?})", expr, ast)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_resolver() {
        assert_eq!(Expr("n", Some(N)).resolve(42), 42);
    }
}
