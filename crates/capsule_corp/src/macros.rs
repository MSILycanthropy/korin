//! Internal macros.

/// Define an enum with `from_name` parsing.
///
/// Supports an optional `@custom` marker at the end to add a `Custom(String)`
/// variant that matches `--*` names.
///
/// ```ignore
/// keyword_enum! {
///     pub enum Display {
///         Block = "block",
///         Flex = "flex",
///         None = "none",
///     }
/// }
///
/// keyword_enum! {
///     pub enum Property {
///         Color = "color",
///         Display = "display",
///         @custom  // Adds Custom(String) variant
///     }
/// }
/// // Generates Property::Custom(String) variant
/// // Property::from_name("--foo") => Some(Property::Custom("foo".into()))
/// ```
macro_rules! keyword_enum {
    // Version WITH @custom
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $str:literal
            ),* $(,)?
            @custom
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
            Custom($crate::Pose),
        }

        impl $name {
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($str => Some(Self::$variant),)*
                    _ if name.starts_with("--") => {
                        Some(Self::Custom($crate::Pose::from(&name[2..])))
                    }
                    _ => None,
                }
            }

            pub fn to_name(&self) -> std::borrow::Cow<'static, str> {
                match self {
                    $(Self::$variant => std::borrow::Cow::Borrowed($str),)*
                    Self::Custom(name) => std::borrow::Cow::Owned(format!("--{}", name)),
                }
            }

            pub const fn is_custom(&self) -> bool {
                matches!(self, Self::Custom(_))
            }

            pub const fn as_custom(&self) -> Option<&$crate::Pose> {
                match self {
                    Self::Custom(name) => Some(name),
                    _ => None,
                }
            }
        }
    };

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
