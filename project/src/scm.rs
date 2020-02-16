use crate::bytecode::{Closure, CodeObject, VirtualMachine};
use crate::continuation::{Continuation, ExitProcedure};
use crate::error::{Result, TypeError};
use crate::primitive::RuntimePrimitive;
use crate::sexpr::{Sexpr, TrackedSexpr};
use crate::symbol::Symbol;
use std::cell::Cell;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone)]
pub enum Scm {
    Undefined,
    Uninitialized,
    Nil,
    True,
    False,
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    String(&'static str),
    Vector(&'static [Cell<Scm>]),

    Pair(&'static (Cell<Scm>, Cell<Scm>)),

    Closure(&'static Closure),
    Primitive(RuntimePrimitive),
    Continuation(&'static Continuation),
    ExitProc(&'static ExitProcedure),

    Cell(&'static Cell<Scm>),
}

impl Scm {
    pub fn uninitialized() -> Self {
        Scm::Uninitialized
    }

    pub fn nil() -> Self {
        Self::Nil
    }

    pub fn bool(b: bool) -> Self {
        match b {
            true => Scm::True,
            false => Scm::False,
        }
    }

    pub fn boxed(x: Scm) -> Scm {
        Scm::Cell(Box::leak(Box::new(Cell::new(x))))
    }

    pub fn cons(car: Scm, cdr: Scm) -> Scm {
        Scm::Pair(Box::leak(Box::new((Cell::new(car), Cell::new(cdr)))))
    }

    pub fn closure(func: &'static CodeObject, free_vars: impl Into<Box<[Scm]>>) -> Self {
        Scm::Closure(Box::leak(Box::new(Closure::new(func, free_vars.into()))))
    }

    pub fn symbol(s: impl Into<Symbol>) -> Self {
        Scm::Symbol(s.into())
    }

    pub fn string(s: impl Into<Box<str>>) -> Self {
        Scm::String(Box::leak(s.into()))
    }

    pub fn list<T, I>(items: T) -> Self
    where
        T: IntoIterator<Item = Scm, IntoIter = I>,
        I: DoubleEndedIterator<Item = Scm>,
    {
        let mut out = Scm::Nil;
        for x in items.into_iter().rev() {
            out = Scm::cons(x, out);
        }
        out
    }

    pub fn vector(items: impl IntoIterator<Item = Scm>) -> Self {
        let v: Vec<Cell<Scm>> = items.into_iter().map(Cell::new).collect();
        let static_data = Box::leak(v.into_boxed_slice());
        Scm::Vector(static_data)
    }

    pub fn primitive(proc: RuntimePrimitive) -> Self {
        Scm::Primitive(proc)
    }

    pub fn is_undefined(&self) -> bool {
        match self {
            Scm::Undefined => true,
            _ => false,
        }
    }

    pub fn is_uninitialized(&self) -> bool {
        match self {
            Scm::Uninitialized => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Scm::Nil => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Scm::True | Scm::False => true,
            _ => false,
        }
    }

    pub fn is_false(&self) -> bool {
        match self {
            Scm::False => true,
            _ => false,
        }
    }

    pub fn is_procedure(&self) -> bool {
        match self {
            Scm::Closure(_) | Scm::Primitive(_) => true,
            _ => false,
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            Scm::Primitive(_) => true,
            _ => false,
        }
    }

    pub fn is_cell(&self) -> bool {
        match self {
            Scm::Cell(_) => true,
            _ => false,
        }
    }

    pub fn as_int(&self) -> Result<i64> {
        match self {
            Scm::Int(i) => Ok(*i),
            _ => Err(TypeError::NoInt.into()),
        }
    }

    pub fn as_symbol(&self) -> Result<Symbol> {
        match self {
            Scm::Symbol(s) => Ok(*s),
            _ => Err(TypeError::NoSymbol.into()),
        }
    }

    pub fn as_string(&self) -> Result<&'static str> {
        match self {
            Scm::String(s) => Ok(*s),
            _ => Err(TypeError::NoString(*self).into()),
        }
    }

    pub fn is_pair(&self) -> bool {
        match self {
            Scm::Pair(_) => true,
            _ => false,
        }
    }

    pub fn car(&self) -> Result<Scm> {
        match self {
            Scm::Pair(p) => Ok(p.0.get()),
            _ => Err(TypeError::NoPair(*self).into()),
        }
    }

    pub fn cdr(&self) -> Result<Scm> {
        match self {
            Scm::Pair(p) => Ok(p.1.get()),
            _ => Err(TypeError::NoPair(*self).into()),
        }
    }

    pub fn set_car(&self, x: Scm) -> Result<Scm> {
        match self {
            Scm::Pair(p) => {
                p.0.set(x);
                Ok(Scm::Undefined)
            }
            _ => Err(TypeError::NoPair(*self).into()),
        }
    }

    pub fn set_cdr(&self, x: Scm) -> Result<Scm> {
        match self {
            Scm::Pair(p) => {
                p.1.set(x);
                Ok(Scm::Undefined)
            }
            _ => Err(TypeError::NoPair(*self).into()),
        }
    }

    pub fn vector_ref(&self, idx: usize) -> Result<Scm> {
        match self {
            Scm::Vector(v) => v
                .get(idx)
                .map(Cell::get)
                .ok_or(TypeError::OutOfBounds.into()),
            _ => Err(TypeError::NoVector.into()),
        }
    }

    pub fn as_closure(&self) -> Result<&'static Closure> {
        match self {
            Scm::Closure(cls) => Ok(*cls),
            _ => Err(TypeError::NoClosure.into()),
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        use Scm::*;
        match (self, other) {
            (Nil, Nil) => true,
            (True, True) => true,
            (False, False) => true,
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Symbol(a), Symbol(b)) => a.ptr_eq(b),
            (String(a), String(b)) => *a as *const str == *b as *const str,
            (Vector(a), Vector(b)) => *a as *const _ == *b as *const _,
            (Pair(a), Pair(b)) => *a as *const _ == *b as *const _,
            (Primitive(a), Primitive(b)) => a == b,
            (Cell(a), Cell(b)) => *a as *const _ == *b as *const _,
            _ => false,
        }
    }

    pub fn equals(&self, other: &Self) -> bool {
        use Scm::*;
        match (self, other) {
            (Nil, Nil) => true,
            (True, True) => true,
            (False, False) => true,
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Vector(a), Vector(b)) => a.iter().zip(*b).all(|(a, b)| a.get().equals(&b.get())),
            (Pair(a), Pair(b)) => a.0.get().equals(&b.0.get()) && a.1.get().equals(&b.1.get()),
            (Primitive(a), Primitive(b)) => a == b,
            (Cell(a), Cell(b)) => a.get().equals(&b.get()),
            _ => false,
        }
    }

    pub fn num_less(&self, other: &Self) -> Result<bool> {
        use Scm::*;
        match (*self, *other) {
            (Int(a), Int(b)) => Ok(a < b),
            (Int(a), Float(b)) => Ok((a as f64) < b),
            (Float(a), Int(b)) => Ok(a < (b as f64)),
            (Float(a), Float(b)) => Ok(a < b),
            _ => Err(TypeError::WrongType.into()),
        }
    }

    pub fn set(&self, value: Scm) -> Result<()> {
        match self {
            Scm::Cell(x) => Ok(x.set(value)),
            _ => Err(TypeError::WrongType.into()),
        }
    }

    pub fn get(&self) -> Result<Scm> {
        match self {
            Scm::Cell(x) => Ok(x.get()),
            _ => Err(TypeError::WrongType.into()),
        }
    }

    pub fn invoke(&self, nargs: usize, vm: &mut VirtualMachine) -> Result<()> {
        match self {
            Scm::Closure(cls) => cls.invoke(nargs, vm),
            Scm::Primitive(func) => func.invoke(nargs, vm)?,
            Scm::Continuation(cnt) => cnt.invoke(nargs, vm)?,
            Scm::ExitProc(cnt) => cnt.invoke(nargs, vm)?,
            _ => return Err(TypeError::NotCallable(*self).into()),
        }
        Ok(())
    }

    pub fn invoke_tail(&self, nargs: usize, vm: &mut VirtualMachine) -> Result<()> {
        match self {
            Scm::Closure(cls) => cls.invoke_tail(nargs, vm),
            Scm::Primitive(func) => func.invoke_tail(nargs, vm)?,
            Scm::Continuation(cnt) => cnt.invoke_tail(nargs, vm)?,
            Scm::ExitProc(cnt) => cnt.invoke_tail(nargs, vm)?,
            _ => return Err(TypeError::NotCallable(*self).into()),
        }
        Ok(())
    }

    pub fn caar(&self) -> Result<Scm> {
        self.car()?.car()
    }

    pub fn cadr(&self) -> Result<Scm> {
        self.cdr()?.car()
    }

    pub fn cdar(&self) -> Result<Scm> {
        self.car()?.cdr()
    }

    pub fn cddr(&self) -> Result<Scm> {
        self.cdr()?.cdr()
    }
}

impl std::fmt::Display for Scm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Scm::Undefined => write!(f, "*undefined*"),
            Scm::Uninitialized => write!(f, "*uninitialized*"),
            Scm::Nil => write!(f, "'()"),
            Scm::True => write!(f, "#t"),
            Scm::False => write!(f, "#f"),
            Scm::Int(i) => write!(f, "{}", i),
            Scm::Float(x) => write!(f, "{}", x),
            Scm::Symbol(s) => write!(f, "{}", s),
            Scm::String(s) => write!(f, "{}", s),
            Scm::Vector(v) => {
                write!(f, "#(")?;
                let mut items = v.iter();
                if let Some(x) = items.next() {
                    write!(f, "{}", x.get())?;

                    for x in items {
                        write!(f, " {}", x.get())?;
                    }
                }
                write!(f, ")")
            }
            Scm::Pair(p) => {
                //write!(f, "({} . {})", p.0.get(), p.1.get())
                write!(f, "({}", p.0.get())?;
                let mut cdr = p.1.get();
                loop {
                    match cdr {
                        Scm::Nil => break,
                        Scm::Pair(q) => {
                            write!(f, " {}", q.0.get())?;
                            cdr = q.1.get();
                        }
                        x => {
                            write!(f, " . {}", x)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
            Scm::Closure(cls) => write!(f, "<closure {:p}>", *cls),
            Scm::Primitive(prim) => write!(f, "<primitive {:?}>", prim),
            Scm::Continuation(cnt) => write!(f, "<continuation {:?}>", cnt),
            Scm::ExitProc(cnt) => write!(f, "<exit-procedure {:?}>", cnt),
            Scm::Cell(c) => write!(f, "{}", c.get()),
        }
    }
}

impl From<&Scm> for Scm {
    fn from(scm: &Scm) -> Scm {
        *scm
    }
}

impl From<bool> for Scm {
    fn from(b: bool) -> Scm {
        Scm::bool(b)
    }
}

impl From<i64> for Scm {
    fn from(x: i64) -> Scm {
        Scm::Int(x)
    }
}

impl From<&Sexpr> for Scm {
    fn from(e: &Sexpr) -> Self {
        match e {
            Sexpr::Undefined => Scm::Undefined,
            Sexpr::Uninitialized => Scm::Uninitialized,
            Sexpr::Nil => Scm::Nil,
            Sexpr::True => Scm::True,
            Sexpr::False => Scm::False,
            Sexpr::Int(i) => Scm::Int(*i),
            Sexpr::Float(f) => Scm::Float(*f),
            Sexpr::Symbol(s) => Scm::Symbol(*s),
            Sexpr::String(s) => Scm::string(&**s),
            Sexpr::Pair(p) => Scm::cons((&p.0).into(), (&p.1).into()),
            Sexpr::Vector(v) => {
                let items: Vec<Cell<Scm>> = v.iter().map(|i| Cell::new(i.into())).collect();
                let items = items.into_boxed_slice();
                Scm::Vector(Box::leak(items))
            }
            Sexpr::SyntacticClosure(_) => unimplemented!(),
        }
    }
}

impl From<&TrackedSexpr> for Scm {
    fn from(e: &TrackedSexpr) -> Self {
        (&e.sexpr).into()
    }
}

impl std::ops::Mul for Scm {
    type Output = Result<Scm>;
    fn mul(self, other: Self) -> Self::Output {
        use Scm::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Int(a), Float(b)) => Ok(Float(a as f64 * b)),
            (Float(a), Int(b)) => Ok(Float(a * b as f64)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            _ => Err(TypeError::WrongType.into()),
        }
    }
}

impl std::ops::Div for Scm {
    type Output = Result<Scm>;
    fn div(self, other: Self) -> Self::Output {
        use Scm::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Float(a as f64 / b as f64)),
            (Int(a), Float(b)) => Ok(Float(a as f64 / b)),
            (Float(a), Int(b)) => Ok(Float(a / b as f64)),
            (Float(a), Float(b)) => Ok(Float(a / b)),
            _ => Err(TypeError::WrongType.into()),
        }
    }
}

impl std::ops::Add for Scm {
    type Output = Result<Scm>;
    fn add(self, other: Self) -> Self::Output {
        use Scm::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Int(a), Float(b)) => Ok(Float(a as f64 + b)),
            (Float(a), Int(b)) => Ok(Float(a + b as f64)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            _ => Err(TypeError::WrongType.into()),
        }
    }
}

impl std::ops::Sub for Scm {
    type Output = Result<Scm>;
    fn sub(self, other: Self) -> Self::Output {
        use Scm::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Int(a), Float(b)) => Ok(Float(a as f64 - b)),
            (Float(a), Int(b)) => Ok(Float(a - b as f64)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            _ => Err(TypeError::WrongType.into()),
        }
    }
}

pub trait ResultWrap {
    fn wrap(self) -> Result<Scm>;
}

impl<T> ResultWrap for T
where
    T: Into<Scm>,
{
    fn wrap(self) -> Result<Scm> {
        Ok(self.into())
    }
}

impl<T> ResultWrap for Result<T>
where
    T: Into<Scm>,
{
    fn wrap(self) -> Result<Scm> {
        self.map(T::into)
    }
}

impl ResultWrap for () {
    fn wrap(self) -> Result<Scm> {
        Ok(Scm::Undefined)
    }
}

impl PartialEq for Scm {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl TryFrom<Scm> for usize {
    type Error = crate::error::Error;
    fn try_from(scm: Scm) -> Result<Self> {
        match scm {
            Scm::Int(i) if i >= 0 => Ok(i as usize),
            _ => Err(TypeError::NoPositiveInt(scm).into()),
        }
    }
}

impl TryFrom<&Scm> for usize {
    type Error = crate::error::Error;
    fn try_from(scm: &Scm) -> Result<Self> {
        Self::try_from(*scm)
    }
}
