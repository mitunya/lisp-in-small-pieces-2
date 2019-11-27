use crate::ast::{RuntimePrimitive, RuntimeProcedure};
use crate::env::GlobalRuntimeEnv;
use crate::sexpr::{Sexpr, TrackedSexpr};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Uninitialized,
    Nil,
    True,
    False,
    Int(i64),
    Float(f64),
    String(Rc<str>),
    Vector(Rc<[Value]>),

    Pair(Rc<(Value, Value)>),

    Procedure(RuntimeProcedure),
    Primitive(RuntimePrimitive),

    Box(Box<Value>),
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<Sexpr> for Value {
    fn from(e: Sexpr) -> Self {
        match e {
            Sexpr::Nil => Value::Nil,
            Sexpr::True => Value::True,
            Sexpr::False => Value::False,
            Sexpr::Int(i) => Value::Int(i),
            Sexpr::Float(f) => Value::Float(f),
            Sexpr::String(s) => Value::String(s),
            Sexpr::Vector(v) => {
                let items: Vec<Value> = v.iter().map(|i| i.clone().into()).collect();
                Value::Vector(items.into())
            }
            _ => unimplemented!("{:?}", e),
        }
    }
}

impl From<TrackedSexpr> for Value {
    fn from(e: TrackedSexpr) -> Self {
        e.into_sexpr().into()
    }
}

impl From<RuntimePrimitive> for Value {
    fn from(p: RuntimePrimitive) -> Self {
        Value::Primitive(p)
    }
}

impl Value {
    pub fn nil() -> Self {
        Value::Nil
    }

    pub fn cons(a: Value, b: Value) -> Self {
        Value::Pair(Rc::new((a, b)))
    }

    pub fn is_true(&self) -> bool {
        match self {
            Value::False => false,
            _ => true,
        }
    }

    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::True => Some(true),
            Value::False => Some(false),
            _ => None,
        }
    }

    pub fn is_pair(&self) -> bool {
        self.as_pair().is_some()
    }

    pub fn as_pair(&self) -> Option<&Rc<(Value, Value)>> {
        match self {
            Value::Pair(p) => Some(p),
            _ => None,
        }
    }

    pub fn invoke(&self, args: Vec<Value>, sg: &mut GlobalRuntimeEnv) -> Value {
        match self {
            Value::Procedure(proc) => proc.invoke(args, sg),
            Value::Primitive(proc) => proc.invoke(args),
            _ => panic!("Cannot invoke {:?}", self),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Value::*;
        match self {
            Undefined => Ok(()),
            Uninitialized => write!(f, "*uninitialized*"),
            Nil => write!(f, "'()"),
            True => write!(f, "#t"),
            False => write!(f, "#f"),
            Int(i) => write!(f, "{}", i),
            Float(i) => write!(f, "{}", i),
            String(s) => write!(f, "\"{}\"", s),
            Pair(pair) => {
                let mut pair = pair;
                write!(f, "(")?;
                write!(f, "{}", pair.0)?;
                while let Pair(ref p) = pair.1 {
                    pair = p;
                    write!(f, " {}", pair.0)?;
                }
                if let Nil = pair.1 {
                    write!(f, ")")
                } else {
                    write!(f, " . {})", pair.1)
                }
            }
            Vector(items) => {
                write!(f, "#(")?;
                write!(f, "{}", items[0])?;
                for i in &items[1..] {
                    write!(f, " {}", i)?;
                }
                write!(f, ")")
            }
            Procedure(proc) => write!(f, "{}", proc),
            Primitive(proc) => write!(f, "{}", proc),
            Box(x) => write!(f, "[{}]", x),
        }
    }
}
