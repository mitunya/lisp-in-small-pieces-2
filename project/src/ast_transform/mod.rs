macro_rules! dispatch {

    ($trans:ident on $node:ident: _ => $blk:block) => {$blk};

    ($trans:ident on $node:ident:) => {Visited::Identity};

    ($trans:ident on $node:ident: $kind:ty => $func:expr, $($rest:tt)*) => {
        if let Some(obj) = $node.downcast_ref::<$kind>() {
            $func($trans, obj).into()
        } else {
            dispatch!($trans on $node: $($rest)*)
        }
    };

    ($trans:ident on $node:ident: $var:ident as $kind:ty => $e:expr, $($rest:tt)*) => {
        if let Some($var) = $node.downcast_ref::<$kind>() {
            $e
        } else {
            dispatch!($trans on $node: $($rest)*)
        }
    };
}

pub mod boxify;
pub mod flatten_closures;
pub mod generate_bytecode;
