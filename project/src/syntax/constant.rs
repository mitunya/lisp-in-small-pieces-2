use crate::ast_transform::Transformer;
use crate::scm::Scm;
use crate::sexpr::{Sexpr, TrackedSexpr};
use crate::source::SourceLocation;
use crate::syntax::Reify;

#[derive(Clone)]
pub struct Constant {
    pub value: Sexpr,
    pub span: SourceLocation,
}

impl_sourced!(Constant);

impl Constant {
    pub fn new(value: Sexpr, span: SourceLocation) -> Self {
        Constant { value, span }
    }

    pub fn default_transform(self, _visitor: &mut impl Transformer) -> Self {
        self
    }
}

impl std::fmt::Debug for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.value {
            Sexpr::SyntacticClosure(sc) => write!(f, "Constant(<{}>)", sc.sexpr().sexpr),
            x => write!(f, "Constant({})", x),
        }
    }
}

impl From<TrackedSexpr> for Constant {
    fn from(sexpr: TrackedSexpr) -> Self {
        Constant {
            value: sexpr.sexpr,
            span: sexpr.src,
        }
    }
}

impl From<Sexpr> for Constant {
    fn from(sexpr: Sexpr) -> Self {
        Constant {
            value: sexpr,
            span: SourceLocation::NoSource,
        }
    }
}

impl Reify for Constant {
    fn reify(&self) -> Scm {
        let x: Scm = (&self.value).into();
        match x {
            Scm::Symbol(_) => Scm::list(vec![Scm::symbol("quote"), x]),
            _ => x,
        }
    }
}
