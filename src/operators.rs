use std::{fmt::Debug, fs::DirEntry};

use crate::tokenizer::Criteria;

pub(crate) enum Operators<X> {
    Buf(Box<dyn Fn(&X) -> bool>),
    Not(Box<Operators<X>>),
    And{rhs: Box<Operators<X>>, lhs: Box<Operators<X>>},
    Or{rhs: Box<Operators<X>>, lhs: Box<Operators<X>>},
    Xor{rhs: Box<Operators<X>>, lhs: Box<Operators<X>>},
    Conditional{rhs: Box<Operators<X>>, lhs: Box<Operators<X>>},
    Biconditional{rhs: Box<Operators<X>>, lhs: Box<Operators<X>>},
}

impl<X, F: Fn(&X) -> bool + 'static> From<F> for Operators<X> {
    fn from(value: F) -> Self {
        Self::Buf(Box::new(value))
    }
}

impl TryFrom<&crate::tokenizer::Criteria> for Operators<DirEntry> {
    type Error = String;

    fn try_from(value: &crate::tokenizer::Criteria) -> Result<Self, Self::Error> {
        Ok(match value {
            Criteria::Size(size_constraint) => crate::parser::create_size_constraint(size_constraint)?,
            Criteria::Type(types) => crate::parser::create_type_constraint(types)?,
            Criteria::Name(name) => crate::parser::create_name_constraint(name)?,
            Criteria::Ext(ext) => crate::parser::create_ext_constraint(ext)?,
            Criteria::Perm(perm) => crate::parser::create_perm_constraint(perm)?,
            Criteria::Atime(atime) => crate::parser::create_atime_constraint(atime)?,
            Criteria::Mtime(mtime) => todo!(),
            Criteria::Ctime(ctime) => todo!(),
            Criteria::Misc(misc) => crate::parser::create_misc_constraint(misc)?,
            _ => todo!()
        }.into())
    }
}

impl<X> Operators<X> {
    pub fn buf<F: Fn(&X) -> bool + 'static>(p: F) -> Self {
        Self::from(p)
    }

    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }

    pub fn new_not<P: Into<Self>>(p: P) -> Self {
        Self::Not(Box::new(p.into()))
    }

    pub fn and<P: Into<Self>>(self, rhs: P) -> Self {
        Self::And { rhs: Box::new(rhs.into()), lhs: Box::new(self) }
    }

    pub fn new_and(lhs: Self, rhs: Self) -> Self {
        Self::And { rhs: Box::new(rhs), lhs: Box::new(lhs) }
    }


    pub fn or<P: Into<Self>>(self, rhs: P) -> Self {
        Self::Or { rhs: Box::new(rhs.into()), lhs: Box::new(self) }
    }

    pub fn new_or(lhs: Self, rhs: Self) -> Self {
        Self::Or { rhs: Box::new(rhs), lhs: Box::new(lhs) }
    }

    pub fn xor<P: Into<Self>>(self, rhs: P) -> Self {
        Self::Xor { rhs: Box::new(rhs.into()), lhs: Box::new(self) }
    }

    pub fn new_xor(lhs: Self, rhs: Self) -> Self {
        Self::Xor { rhs: Box::new(rhs), lhs: Box::new(lhs) }
    }

    pub fn conditional<P: Into<Self>>(self, rhs: P) -> Self {
        Self::Conditional { rhs: Box::new(rhs.into()), lhs: Box::new(self) }
    }

    pub fn new_conditional(lhs: Self, rhs: Self) -> Self {
        Self::Conditional { rhs: Box::new(rhs), lhs: Box::new(lhs) }
    }

    pub fn biconditional<P: Into<Self>>(self, rhs: P) -> Self {
        Self::Biconditional { rhs: Box::new(rhs.into()), lhs: Box::new(self) }
    }

    pub fn new_biconditional(lhs: Self, rhs: Self) -> Self {
        Self::Biconditional { rhs: Box::new(rhs), lhs: Box::new(lhs) }
    }

    pub fn eval(&self, e: &X) -> bool {
        match self {
            Operators::Buf(f) => f(e),
            Operators::Not(p) => !p.eval(e),
            Operators::Or{rhs, lhs}=> lhs.eval(e) || rhs.eval(e),
            Operators::And { rhs, lhs } => lhs.eval(e) && rhs.eval(e),
            Operators::Xor { rhs, lhs } => lhs.eval(e) ^ rhs.eval(e),
            Operators::Conditional { rhs, lhs } => ! lhs.eval(e) || rhs.eval(e),
            Operators::Biconditional { rhs, lhs } => lhs.eval(e) == rhs.eval(e),
        }
    }
}