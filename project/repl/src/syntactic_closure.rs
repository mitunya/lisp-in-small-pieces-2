use crate::env::Env;
use crate::objectify::{Result as ObjectifyResult, Translate};
use crate::sexpr::{Sexpr, TrackedSexpr};
use crate::syntax::Expression;
use std::cell::Cell;
use std::rc::Rc;
use sunny_common::Symbol;

#[derive(Debug, Clone)]
pub struct SyntacticClosure {
    sexpr: TrackedSexpr,
    closed_syntactic_environment: Env,
    free_names: Vec<Symbol>,
    bound: Cell<bool>,
}

impl PartialEq for SyntacticClosure {
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!()
    }
}

impl SyntacticClosure {
    pub fn new(sexpr: TrackedSexpr, env: Env) -> Self {
        SyntacticClosure {
            sexpr,
            closed_syntactic_environment: env,
            free_names: vec![],
            bound: Cell::new(false),
        }
    }

    pub fn with_free_names(mut self, names: impl IntoIterator<Item = Symbol>) -> Self {
        self.free_names.extend(names);
        self
    }

    pub fn sexpr(&self) -> &TrackedSexpr {
        &self.sexpr
    }

    pub fn expand(&self, trans: &mut Translate) -> ObjectifyResult<Expression> {
        if self.is_bound() {
            return trans.objectify(&self.sexpr);
        }

        let mut combined_env = self.closed_syntactic_environment.clone();
        for name in &self.free_names {
            let var = trans.env.find_variable(name).unwrap();
            combined_env.push_local(var);
        }

        let tmp = std::mem::replace(&mut trans.env, combined_env);
        let result = trans.objectify(&self.sexpr);
        trans.env = tmp;
        result
    }

    pub fn is_alias(&self) -> bool {
        self.sexpr.is_identifier()
    }

    pub fn is_bound(&self) -> bool {
        self.bound.get()
    }

    pub fn set_bound(&self, b: bool) {
        self.bound.set(b)
    }

    pub fn alias_name(&self) -> Option<Symbol> {
        match &self.sexpr.sexpr {
            Sexpr::Symbol(s) => Some(*s),
            Sexpr::SyntacticClosure(sc) => sc.alias_name(),
            _ => None,
        }
    }

    pub fn set_name(&self, name: Symbol) {
        match &self.sexpr.sexpr {
            Sexpr::Symbol(s) => unsafe {
                let s = &mut *(s as *const _ as *mut Symbol);
                *s = name;
            },
            Sexpr::SyntacticClosure(sc) => sc.set_name(name),
            _ => panic!("not an alias"),
        }
    }

    pub fn rename(&self) {
        match &self.sexpr.sexpr {
            Sexpr::Symbol(s) => unsafe {
                let s = &mut *(s as *const _ as *mut Symbol);
                *s = s.as_uninterned();
            },
            Sexpr::SyntacticClosure(sc) => sc.rename(),
            _ => panic!("not an alias"),
        }
    }
}

impl From<Rc<SyntacticClosure>> for TrackedSexpr {
    fn from(sc: Rc<SyntacticClosure>) -> Self {
        TrackedSexpr {
            src: sc.sexpr.src.clone(),
            sexpr: Sexpr::SyntacticClosure(sc),
        }
    }
}

impl From<SyntacticClosure> for TrackedSexpr {
    fn from(sc: SyntacticClosure) -> Self {
        Rc::new(sc).into()
    }
}
