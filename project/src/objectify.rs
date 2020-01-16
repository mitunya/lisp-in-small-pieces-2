use crate::env::Env;
use crate::macro_language::{expand_captured_binding, is_captured_binding};
use crate::sexpr::TrackedSexpr as Sexpr;
use crate::source::SourceLocation;
use crate::source::SourceLocation::NoSource;
use crate::symbol::Symbol;
use crate::syntax::definition::GlobalDefine;
use crate::syntax::{Alternative, Expression, FixLet, Function, GlobalAssignment, GlobalReference, GlobalVariable, LocalAssignment, LocalReference, LocalVariable, MagicKeyword, PredefinedApplication, PredefinedReference, PredefinedVariable, Reference, RegularApplication, Sequence, Variable, NoOp};
use crate::utils::Sourced;
use std::convert::TryInto;
use crate::library::{is_import, Library};
use crate::objectify::ObjectifyErrorKind::UnknownLibrary;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub type Result<T> = std::result::Result<T, ObjectifyError>;

#[derive(Debug)]
pub struct ObjectifyError {
    pub kind: ObjectifyErrorKind,
    pub location: SourceLocation,
}

#[derive(Debug, PartialEq)]
pub enum ObjectifyErrorKind {
    SyntaxError,
    NoPair,
    IncorrectArity,
    ImmutableAssignment,
    ExpectedList,
    ExpectedSymbol,
    UnknownLibrary(PathBuf),
}

#[derive(Debug)]
pub struct Translate {
    pub env: Env,
    pub libs: HashMap<PathBuf, Library>,
}

impl Translate {
    pub fn new(env: Env) -> Self {
        Translate { env,
        libs: HashMap::new()}
    }

    pub fn objectify_toplevel(&mut self, exprs: &[Sexpr]) -> Result<Expression> {
        let n_imports = exprs.iter().take_while(|&expr| is_import(expr)).count();

        for expr in exprs[..n_imports].iter() {
            self.import(expr.cdr().unwrap())?;
        }

        let mut sequence = Sexpr::nil(NoSource);
        for expr in exprs[n_imports..].iter().rev() {
            sequence = Sexpr::cons(expr.clone(), sequence, NoSource);
        }
        sequence = Sexpr::cons(Sexpr::symbol("begin", NoSource), sequence, NoSource);
        self.objectify(&sequence, &self.env.clone())
    }

    pub fn objectify(&mut self, expr: &Sexpr, env: &Env) -> Result<Expression> {
        if expr.is_atom() {
            match () {
                _ if expr.is_symbol() => self.objectify_symbol(expr, env),
                _ => self.objectify_quotation(expr, env),
            }
        } else if is_captured_binding(expr) {
            expand_captured_binding(self, expr, env)
        } else {
            let m = self.objectify(ocar(expr)?, env)?;
            if let Expression::MagicKeyword(MagicKeyword { name: _, handler }) = m {
                handler(self, expr, env)
            } else {
                self.objectify_application(&m, ocdr(expr)?, env, expr.source().clone())
            }
        }
    }

    fn import(&mut self, import_set: &Sexpr) -> Result<Expression> {
        let library_name = import_set.car().unwrap();
        let lib = self.library(library_name)?;
        lib.import_into_environment(&mut self.env);
        Ok(unimplemented!())
    }

    pub fn objectify_quotation(&mut self, expr: &Sexpr, _env: &Env) -> Result<Expression> {
        Ok(Expression::Constant(expr.clone().into()))
    }

    pub fn objectify_alternative(
        &mut self,
        condition: &Sexpr,
        consequence: &Sexpr,
        alternative: Option<&Sexpr>,
        env: &Env,
        span: SourceLocation,
    ) -> Result<Expression> {
        let condition = self.objectify(condition, env)?;
        let consequence = self.objectify(consequence, env)?;
        let alternative = match alternative {
            Some(alt) => self.objectify(alt, env)?,
            None => Expression::Constant(Sexpr::undefined().into()),
        };
        Ok(Alternative::new(condition, consequence, alternative, span).into())
    }

    pub fn objectify_sequence(&mut self, exprs: &Sexpr, env: &Env) -> Result<Expression> {
        if exprs.is_pair() {
            let car = exprs.car().unwrap();
            let cdr = exprs.cdr().unwrap();
            if cdr.is_pair() {
                let this = self.objectify(car, env)?;
                let next = self.objectify_sequence(cdr, env)?;
                Ok(Sequence::new(this, next, exprs.src.clone()).into())
            } else {
                self.objectify(car, env)
            }
        } else {
            Err(ObjectifyError {
                kind: ObjectifyErrorKind::ExpectedList,
                location: exprs.src.clone(),
            })
        }
    }

    fn objectify_symbol(&mut self, expr: &Sexpr, env: &Env) -> Result<Expression> {
        let var_name = Sexpr::as_symbol(expr).unwrap();
        match env.find_variable(var_name) {
            Some(Variable::LocalVariable(v)) => {
                Ok(LocalReference::new(v, expr.source().clone()).into())
            }
            Some(Variable::GlobalVariable(v)) => {
                Ok(GlobalReference::new(v, expr.source().clone()).into())
            }
            Some(Variable::PredefinedVariable(v)) => {
                Ok(PredefinedReference::new(v, expr.source().clone()).into())
            }
            Some(Variable::MagicKeyword(mkw)) => Ok((mkw).into()),
            Some(Variable::FreeVariable(_)) => {
                panic!("There should be no free variables in the compile-time environment")
            }
            None => self.objectify_free_reference(var_name.clone(), env, expr.source().clone()),
        }
    }

    fn objectify_free_reference(
        &mut self,
        name: Symbol,
        env: &Env,
        span: SourceLocation,
    ) -> Result<Expression> {
        let v = self.adjoin_global_variable(name, env);
        Ok(GlobalReference::new(v, span).into())
    }

    fn adjoin_global_variable(&mut self, name: Symbol, env: &Env) -> GlobalVariable {
        let v = GlobalVariable::new(name);
        env.globals.extend(v.clone().into());
        v
    }

    fn objectify_application(
        &mut self,
        func: &Expression,
        mut args: &Sexpr,
        env: &Env,
        span: SourceLocation,
    ) -> Result<Expression> {
        let mut args_list = vec![];
        while !args.is_null() {
            let car = args.car().ok_or_else(|| ObjectifyError {
                kind: ObjectifyErrorKind::ExpectedList,
                location: args.source().clone(),
            })?;
            args_list.push(self.objectify(car, env)?);
            args = args.cdr().unwrap();
        }

        match func {
            Expression::Function(f) => self.process_closed_application(f.clone(), args_list, span),
            Expression::Reference(Reference::PredefinedReference(p)) => {
                let fvf = p.var.clone();
                let desc = fvf.description();
                if desc.arity.check(args_list.len()) {
                    Ok(PredefinedApplication::new(fvf, args_list, span).into())
                } else {
                    Err(ObjectifyError {
                        kind: ObjectifyErrorKind::IncorrectArity,
                        location: span,
                    })
                }
            }
            _ => Ok(RegularApplication::new(func.clone(), args_list, span).into()),
        }
    }

    fn process_closed_application(
        &mut self,
        func: Function,
        args: Vec<Expression>,
        span: SourceLocation,
    ) -> Result<Expression> {
        if func
            .variables
            .last()
            .map(LocalVariable::is_dotted)
            .unwrap_or(false)
        {
            self.process_nary_closed_application(func, args, span)
        } else {
            if args.len() == func.variables.len() {
                Ok(FixLet::new(func.variables, args, func.body, span).into())
            } else {
                Err(ObjectifyError {
                    kind: ObjectifyErrorKind::IncorrectArity,
                    location: span,
                })
            }
        }
    }

    fn process_nary_closed_application(
        &mut self,
        func: Function,
        mut args: Vec<Expression>,
        span: SourceLocation,
    ) -> Result<Expression> {
        let variables = func.variables;
        let body = func.body;

        if args.len() + 1 < variables.len() {
            return Err(ObjectifyError {
                kind: ObjectifyErrorKind::IncorrectArity,
                location: span,
            });
        }

        let cons_var: PredefinedVariable = self
            .env
            .predef
            .find_variable("cons")
            .expect("The cons pritimive must be available in the predefined environment")
            .try_into()
            .unwrap();

        let mut dotted = Expression::Constant(Sexpr::nil(span.last_char()).into());

        while args.len() >= variables.len() {
            let x = args.pop().unwrap();

            let partial_span = span.clone().start_at(x.source());

            dotted =
                PredefinedApplication::new(cons_var.clone(), vec![x, dotted], partial_span).into();
        }

        args.push(dotted.into());

        variables.last().unwrap().set_dotted(false);

        Ok(FixLet::new(variables, args, body, span).into())
    }

    pub fn objectify_function(
        &mut self,
        names: &Sexpr,
        body: &Sexpr,
        env: &Env,
        span: SourceLocation,
    ) -> Result<Expression> {
        let vars = self.objectify_variables_list(names)?;
        env.locals.extend_frame(vars.iter().cloned());
        let bdy = self.objectify_sequence(body, env)?;
        env.locals.pop_frame(vars.len());
        Ok(Function::new(vars, bdy, span).into())
    }

    fn objectify_variables_list(&mut self, mut names: &Sexpr) -> Result<Vec<LocalVariable>> {
        let mut vars = vec![];
        while let Some(car) = names.car() {
            let name = car.as_symbol().ok_or_else(|| ObjectifyError {
                kind: ObjectifyErrorKind::ExpectedSymbol,
                location: car.source().clone(),
            })?;
            vars.push(LocalVariable::new(*name, false, false));

            names = names.cdr().unwrap();
        }

        if !names.is_null() {
            let name = names.as_symbol().ok_or_else(|| ObjectifyError {
                kind: ObjectifyErrorKind::ExpectedSymbol,
                location: names.source().clone(),
            })?;
            vars.push(LocalVariable::new(*name, false, true));
        }

        Ok(vars)
    }

    pub fn objectify_definition(
        &mut self,
        variable: &Sexpr,
        expr: &Sexpr,
        env: &Env,
        span: SourceLocation,
    ) -> Result<Expression> {
        let form = self.objectify(expr, env)?;

        let var_name = Sexpr::as_symbol(variable).unwrap();
        let gvar = match env.find_variable(var_name) {
            Some(Variable::LocalVariable(_)) => panic!("untransformed local define"),
            Some(Variable::GlobalVariable(v)) => v,
            _ => self.adjoin_global_variable(*var_name, env),
        };

        Ok(GlobalDefine::new(gvar, form, span).into())
    }

    pub fn objectify_assignment(
        &mut self,
        variable: &Sexpr,
        expr: &Sexpr,
        env: &Env,
        span: SourceLocation,
    ) -> Result<Expression> {
        let ov = self.objectify_symbol(variable, env)?;
        let of = self.objectify(expr, env)?;

        match ov {
            Expression::Reference(Reference::LocalReference(r)) => {
                r.var.set_mutable(true);
                Ok(LocalAssignment::new(r, of, span).into())
            }
            Expression::Reference(Reference::GlobalReference(r)) => {
                Ok(GlobalAssignment::new(r.var, of, span).into())
            }
            _ => Err(ObjectifyError {
                kind: ObjectifyErrorKind::ImmutableAssignment,
                location: span,
            }),
        }
    }

    fn library(&mut self, library_name: &Sexpr) -> Result<Library> {
        let mut path = PathBuf::new();

        let mut next_part = library_name;
        while next_part.is_pair() {
            path.push(format!("{}", next_part.car().unwrap()));
            next_part = next_part.cdr().unwrap();
        }

        match self.libs.entry(path) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                Err(ObjectifyError{
                    kind: ObjectifyErrorKind::UnknownLibrary(entry.into_key()).into(),
                    location: library_name.src.clone()
                })
            }
        }
    }

    pub fn add_library(&mut self, library_name: impl Into<PathBuf>, library: Library) {
        self.libs.insert(library_name.into(), library);
    }
}

pub fn ocar(e: &Sexpr) -> Result<&Sexpr> {
    e.car().ok_or_else(|| ObjectifyError {
        kind: ObjectifyErrorKind::NoPair,
        location: e.source().clone(),
    })
}

pub fn ocdr(e: &Sexpr) -> Result<&Sexpr> {
    e.cdr().ok_or_else(|| ObjectifyError {
        kind: ObjectifyErrorKind::NoPair,
        location: e.source().clone(),
    })
}

pub fn decons(e: &Sexpr) -> Result<(&Sexpr, &Sexpr)> {
    ocar(e).and_then(|car| ocdr(e).map(|cdr| (car, cdr)))
}
