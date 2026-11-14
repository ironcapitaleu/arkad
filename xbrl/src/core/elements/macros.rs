//! # Concept Macros
//!
//! Provides the two macros that define the enums of the concept hierarchy.
//!
//! - `innermost_concepts!` defines one innermost enum, such as `BalanceSheetConcept`, from one
//!   list of variants and labels.
//! - `middle_concepts!` defines the middle enums, `StatementConcept` and `DisclosureConcept`, the
//!   conversions between the levels, and `INNERMOST`.
//!
//! One list feeds each enum and its tables, so a variant cannot miss `ALL`, `ELEMENTS`, `name()`,
//! or `INNERMOST`.

/// Defines an innermost concept enum and its `impl Concept`.
///
/// The input names the enum, the variant of `CanonicalElementKind` and of the middle enum that
/// hold it, and each variant with its label. The macro writes:
///
/// - the enum, `#[non_exhaustive]`, with the derives of every concept enum,
/// - `ELEMENTS`, the `CanonicalElementKind` of each variant, in the order of the list,
/// - the inherent `const fn element()`,
/// - `impl Concept` with `ALL` in the order of the list, `name()` from `stringify!`, and `label()`.
macro_rules! innermost_concepts {
    (
        $(#[$meta:meta])*
        pub enum $name:ident in $top:ident($middle:ident::$middle_variant:ident) {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $label:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[non_exhaustive]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl $name {
            /// The standard concept of each variant, in the order of [`Concept::ALL`].
            ///
            /// [`Concept::ALL`]: crate::core::elements::Concept::ALL
            pub const ELEMENTS: &'static [$crate::core::elements::CanonicalElementKind] =
                &[$(Self::$variant.element(),)+];

            /// Returns the standard concept that holds this variant.
            ///
            /// It is a `const fn`, so a `const` table, such as an alias list, can name a
            /// standard concept.
            #[must_use]
            pub const fn element(self) -> $crate::core::elements::CanonicalElementKind {
                $crate::core::elements::CanonicalElementKind::$top(
                    $crate::core::elements::$middle::$middle_variant(self),
                )
            }
        }

        impl $crate::core::elements::Concept for $name {
            const ALL: &'static [Self] = &[$(Self::$variant,)+];

            fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)+
                }
            }

            fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }
        }
    };
}

/// Defines the middle concept enums, the conversions between the levels, and `INNERMOST`.
///
/// Each middle enum names the variant of `CanonicalElementKind` that holds it, and each of its
/// variants names one innermost enum. For each middle enum, the macro writes:
///
/// - the enum, `#[non_exhaustive]`, with the derives of every concept enum,
/// - `name()` and `label()`, from the [`Concept`](crate::core::elements::Concept) of the
///   innermost enum,
/// - `From` one level up and `TryFrom` one level down, and `TryFrom` from `CanonicalElementKind`
///   to each innermost enum.
///
/// It also writes `INNERMOST`, with the `ELEMENTS` of each innermost enum in the order of the
/// input.
macro_rules! middle_concepts {
    (
        $(
            $(#[$meta:meta])*
            pub enum $middle:ident in $top:ident {
                $(
                    $(#[$variant_meta:meta])*
                    $variant:ident($inner:ident)
                ),+ $(,)?
            }
        )+
    ) => {
        $(
            $(#[$meta])*
            #[non_exhaustive]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum $middle {
                $(
                    $(#[$variant_meta])*
                    $variant($inner),
                )+
            }

            impl $middle {
                /// Returns the name in silver of the concept, from
                /// [`Concept::name`](crate::core::elements::Concept::name).
                #[must_use]
                pub fn name(self) -> &'static str {
                    match self {
                        $(Self::$variant(concept) => $crate::core::elements::Concept::name(concept),)+
                    }
                }

                /// Returns the label for people of the concept, from
                /// [`Concept::label`](crate::core::elements::Concept::label).
                #[must_use]
                pub fn label(self) -> &'static str {
                    match self {
                        $(Self::$variant(concept) => $crate::core::elements::Concept::label(concept),)+
                    }
                }
            }

            impl From<$middle> for $crate::core::elements::CanonicalElementKind {
                fn from(concept: $middle) -> Self {
                    Self::$top(concept)
                }
            }

            impl TryFrom<$crate::core::elements::CanonicalElementKind> for $middle {
                type Error = $crate::core::elements::CanonicalElementKind;

                /// # Errors
                ///
                #[doc = concat!("Returns the element as it is if it holds no [`", stringify!($middle), "`].")]
                fn try_from(element: $crate::core::elements::CanonicalElementKind) -> Result<Self, Self::Error> {
                    match element {
                        $crate::core::elements::CanonicalElementKind::$top(concept) => Ok(concept),
                        other => Err(other),
                    }
                }
            }

            $(
                impl From<$inner> for $middle {
                    fn from(concept: $inner) -> Self {
                        Self::$variant(concept)
                    }
                }

                impl TryFrom<$middle> for $inner {
                    type Error = $middle;

                    /// # Errors
                    ///
                    #[doc = concat!("Returns the concept as it is if it is no [`", stringify!($inner), "`].")]
                    // A middle enum with one variant leaves the arm `other` unreachable. The macro
                    // writes it for every middle enum, so a new variant needs no change here.
                    #[allow(unreachable_patterns)]
                    fn try_from(concept: $middle) -> Result<Self, Self::Error> {
                        match concept {
                            $middle::$variant(inner) => Ok(inner),
                            other => Err(other),
                        }
                    }
                }

                impl TryFrom<$crate::core::elements::CanonicalElementKind> for $inner {
                    type Error = $crate::core::elements::CanonicalElementKind;

                    /// # Errors
                    ///
                    #[doc = concat!("Returns the element as it is if it holds no [`", stringify!($inner), "`].")]
                    fn try_from(element: $crate::core::elements::CanonicalElementKind) -> Result<Self, Self::Error> {
                        $middle::try_from(element).and_then(|concept| {
                            Self::try_from(concept).map_err($crate::core::elements::CanonicalElementKind::from)
                        })
                    }
                }
            )+
        )+

        /// The standard concepts of each innermost enum, one entry per enum.
        ///
        /// The entries follow the order of the middle enums. The parse from the name and the test
        /// for unique names walk this table.
        pub const INNERMOST: &[&[$crate::core::elements::CanonicalElementKind]] = &[$($($inner::ELEMENTS,)+)+];
    };
}
