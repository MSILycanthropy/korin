//! Internal macros.

/// Define an enum with `from_name` parsing.
///
/// ```ignore
/// keyword_enum! {
///     pub enum Display {
///         Block = "block",
///         Flex = "flex",
///         #[default]
///         None = "none",
///     }
/// }
/// ```
macro_rules! keyword_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $str:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $name {
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($str => Some(Self::$variant),)*
                    _ => None,
                }
            }

            pub const fn to_name(self) -> &'static str {
                match self {
                    $(Self::$variant => $str,)*
                }
            }
        }
    };
}

pub(crate) use keyword_enum;
