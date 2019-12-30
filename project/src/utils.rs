/// Shortcut to declare enums of the Form `enum Foo { Bar(Bar), Baz(Baz) }` along with
/// implementations of `impl From<Bar> for Foo` and `impl TryFrom<Foo> for Bar` for each variant.
macro_rules! sum_type {
    // declare enum with decorations such as `#[derive(Debug)]`
    (#[$($attrs:tt)*] type $typename:ident = $($type:ident)|+;) => {
        #[$($attrs)*]
        enum $typename {
            $(
                $type($type)
            ),*
        }
        sum_type! {@impls  $typename $($type)*}
    };

    // declare enum
    (type $typename:ident = $($type:ident)|+;) => {
        enum $typename {
            $(
                $type($type)
            ),*
        }
        sum_type! {@impls  $typename $($type)*}
    };

    // declare enum with decorations such as `#[derive(Debug)]`
    (#[$($attrs:tt)*] pub type $typename:ident = $($type:ident)|+;) => {
        #[$($attrs)*]
        pub enum $typename {
            $(
                $type($type)
            ),*
        }
        sum_type! {@impls  $typename $($type)*}
    };

    // declare enum
    (pub type $typename:ident = $($type:ident)|+;) => {
        pub enum $typename {
            $(
                $type($type)
            ),*
        }
        sum_type! {@impls  $typename $($type)*}
    };

    // delegate trait implementation for each variant
    (@impls $typename:ident $($type:ident)+) => {
        $(
            sum_type! { @impl-from $typename $type }
            sum_type! { @impl-try_from $typename $type }
        )*
    };

    // implement `From` trait for one variant
    (@impl-from $typename:ident $type:ident) => {
        impl From<$type> for $typename {
            fn from(x: $type) -> Self {
                $typename::$type(x)
            }
        }
    };

    // implement `TryFrom` trait for one variant
    (@impl-try_from $typename:ident $type:ident) => {
        impl ::std::convert::TryFrom<$typename> for $type {
            type Error = $typename;
            fn try_from(x: $typename) -> Result<$type, Self::Error> {
                match x {
                    $typename::$type(value) => Ok(value),
                    _ => Err(x),
                }
            }
        }
    };

    // implement delegated method
    (@impl-method $typename:ident $($type:ident)* $method:ident(&self, $($args:ident : $argty:ty),*) -> $retty:ty) => {
        impl $typename {
            fn $method(&self, $($args : $argty),*) -> $retty {
                match self {

                }
            }
        }
    };
}

/// Shortcut to declare multiple enums of the Form `enum Foo { Bar(Bar), Baz(Baz) }` along with
/// implementations of `impl From<Bar> for Foo` and `impl TryFrom<Foo> for Bar` for each variant.
macro_rules! sum_types {
    () => {};

    (#[$($attrs:tt)*] type $typename:ident = $($type:ident)|+; $($rest:tt)*) => {
        sum_type! {
            #[$($attrs)*]
            type $typename = $($type)|+;
        }
        sum_types! { $($rest)* }
    };

    (type $typename:ident = $($type:ident)|+; $($rest:tt)*) => {
        sum_type! {
            type $typename = $($type)|+;
        }
        sum_types! { $($rest)* }
    };

    (#[$($attrs:tt)*] pub type $typename:ident = $($type:ident)|+; $($rest:tt)*) => {
        sum_type! {
            #[$($attrs)*]
            pub type $typename = $($type)|+;
        }
        sum_types! { $($rest)* }
    };

    (pub type $typename:ident = $($type:ident)|+; $($rest:tt)*) => {
        sum_type! {
            pub type $typename = $($type)|+;
        }
        sum_types! { $($rest)* }
    };
}
