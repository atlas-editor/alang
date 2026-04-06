use std::fmt::{self, Write};

use crate::parse::{Expression, Function, Import, Operator, Program, Statement};

trait ToGolang {
    fn fmtgo<W: Write>(&self, f: &mut W) -> fmt::Result;
}

impl ToGolang for Import {
    fn fmtgo<W: Write>(&self, f: &mut W) -> fmt::Result {
        write!(f, "    \"{}\"", self.0.join("/"))
    }
}

impl ToGolang for Operator {
    fn fmtgo<W: Write>(&self, f: &mut W) -> fmt::Result {
        match &self {
            Operator::Plus => f.write_str("+"),
        }
    }
}

impl ToGolang for Expression {
    fn fmtgo<W: Write>(&self, f: &mut W) -> fmt::Result {
        match self {
            Expression::Identifier(x) => f.write_str(x),
            Expression::IdentifierChain(xs) => {
                for (i, part) in xs.iter().enumerate() {
                    if i > 0 {
                        f.write_str(".")?;
                    }
                    f.write_str(part)?;
                }
                Ok(())
            }
            Expression::String(x) => write!(f, "\"{}\"", x),
            Expression::Float(x) => write!(f, "{}", x),
            Expression::Integer(x) => write!(f, "{}", x),
            Expression::Bool(x) => write!(f, "{}", x),
            Expression::Byte(x) => write!(f, "{}", x),
            Expression::Prefix { op, exp } => {
                op.fmtgo(f)?;
                f.write_str("(")?;
                exp.fmtgo(f)?;
                f.write_str(")")
            }
            Expression::Infix { op, left, right } => {
                f.write_str("(")?;
                left.fmtgo(f)?;
                f.write_str(")")?;
                op.fmtgo(f)?;
                f.write_str("(")?;
                right.fmtgo(f)?;
                f.write_str(")")
            }
            Expression::Call { caller, args } => {
                caller.fmtgo(f)?;
                f.write_str("(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    arg.fmtgo(f)?;
                }
                f.write_str(")")
            }
        }
    }
}

impl ToGolang for Statement {
    fn fmtgo<W: Write>(&self, f: &mut W) -> fmt::Result {
        match self {
            Statement::Assignment {
                identifier,
                expression,
            } => {
                write!(f, "{} := ", identifier)?;
                expression.fmtgo(f)?;
                write!(f, ";")
            }
            Statement::Return(expression) => {
                write!(f, "return ")?;
                expression.fmtgo(f)?;
                write!(f, ";")
            }
            Statement::Reassignment {
                identifier,
                expression,
            } => {
                write!(f, "{} = ", identifier)?;
                expression.fmtgo(f)?;
                write!(f, ";")
            }
            Statement::Expression(expression) => {
                expression.fmtgo(f)?;
                write!(f, ";")
            }
        }
    }
}

impl ToGolang for Function {
    fn fmtgo<W: Write>(&self, f: &mut W) -> fmt::Result {
        todo!()
    }
}

impl ToGolang for Program {
    fn fmtgo<W: Write>(&self, f: &mut W) -> fmt::Result {
        writeln!(f, "import (")?;
        for i in &self.imports {
            i.fmtgo(f)?;
            writeln!(f)?;
        }
        writeln!(f, ")")?;
        for func in &self.functions {
            func.fmtgo(f)?;
            writeln!(f, "\n")?;
        }

        Ok(())
    }
}
