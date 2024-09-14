//! Convert built-in scalar types.

use alloc::{format, string::String};
use core::str::FromStr;

use repr::{wrappers::*, Repr};

use crate::{
    ast::Scalar,
    context::Context,
    errors::{DecodeError, EncodeError, ExpectedType},
    traits::{DecodeScalar, EncodeScalar},
};

fn digit(radix: u32) -> Repr<char> {
    match radix {
        2 => interval('0', '1'),
        8 => interval('0', '7'),
        10 => interval('0', '9'),
        16 => interval('0', '9')
            .or(interval('a', 'f'))
            .or(interval('A', 'F')),
        _ => panic!("invalid radix"),
    }
}

fn digits(radix: u32) -> Repr<char> {
    one('_').or(digit(radix)).inf()
}

fn decimal_number() -> Repr<char> {
    empty()
        .or(one('-').or(one('+')))
        .mul(digit(10))
        .mul(digits(10))
        .mul(one('.').mul(digit(10)).mul(digits(10)).or(empty()))
        .mul(
            one('e')
                .or(one('E'))
                .mul(empty().or(one('-').or(one('+'))))
                .mul(digits(10))
                .or(empty()),
        )
}

fn radix_number() -> Repr<char> {
    empty().or(one('-').or(one('+'))).mul(one('0')).mul(
        one('b')
            .mul(digit(2).mul(digits(2)))
            .or(one('o').mul(digit(8).mul(digits(8))))
            .or(one('x').mul(digit(16).mul(digits(16)))),
    )
}

fn number(value: &str) -> Option<(u32, String)> {
    match decimal_number().to_regex().captures(value) {
        Some(caps) => Some((
            10,
            caps.get(0)
                .unwrap()
                .as_str()
                .chars()
                .filter(|c| c != &'_')
                .collect::<String>(),
        )),
        None => match radix_number().to_regex().captures(value) {
            Some(caps) => {
                let mut value = caps
                    .get(0)
                    .unwrap()
                    .as_str()
                    .chars()
                    .filter(|c| c != &'_')
                    .collect::<String>();
                let sign = if value.starts_with('-') || value.starts_with('+') {
                    Some(value.remove(0))
                } else {
                    None
                };
                let radix = match &value[..2] {
                    "0b" => 2,
                    "0o" => 8,
                    "0x" => 16,
                    _ => unreachable!(),
                };
                let mut s = String::with_capacity(value.len() + sign.map_or(0, |_| 1));
                if let Some(c) = sign {
                    s.push(c)
                }
                s.extend(value[2..].chars());
                Some((radix, s))
            }
            None => None,
        },
    }
}

macro_rules! impl_integer {
    ($ty:ident) => {
        impl DecodeScalar for $ty {
            fn decode(scalar: &Scalar, ctx: &mut Context) -> Result<Self, DecodeError> {
                if let Some(typ) = scalar.type_name.as_ref() {
                    if typ.as_ref() != stringify!($ty) {
                        return Err(DecodeError::TypeName {
                            span: ctx.span(&typ),
                            found: Some(typ.clone()),
                            expected: ExpectedType::optional(stringify!($ty)),
                            rust_type: stringify!($ty),
                        });
                    }
                }
                match number(scalar.literal.as_ref()) {
                    Some((radix, value)) => <$ty>::from_str_radix(&value, radix)
                        .map_err(|err| DecodeError::conversion(ctx.span(&scalar), err)),
                    None => Err(DecodeError::scalar_kind(
                        ctx.span(&scalar),
                        "integer",
                        scalar.literal.clone(),
                    )), // TODO(rnarkk)
                }
            }
        }

        impl EncodeScalar for $ty {
            fn encode(&self, _: &mut Context) -> Result<Scalar, EncodeError> {
                let literal = format!("{}", self);
                Ok(Scalar {
                    type_name: None,
                    literal: literal.into(),
                })
            }
        }
    };
}

impl_integer!(i8);
impl_integer!(u8);
impl_integer!(i16);
impl_integer!(u16);
impl_integer!(i32);
impl_integer!(u32);
impl_integer!(i64);
impl_integer!(u64);
impl_integer!(isize);
impl_integer!(usize);

macro_rules! impl_decimal {
    ($ty:ident) => {
        impl DecodeScalar for $ty {
            fn decode(scalar: &Scalar, ctx: &mut Context) -> Result<Self, DecodeError> {
                if let Some(typ) = scalar.type_name.as_ref() {
                    if typ.as_ref() != stringify!($ty) {
                        return Err(DecodeError::TypeName {
                            span: ctx.span(&typ),
                            found: Some(typ.clone()),
                            expected: ExpectedType::optional(stringify!($ty)),
                            rust_type: stringify!($ty),
                        });
                    }
                }
                match number(scalar.literal.as_ref()) {
                    Some((10, value)) => <$ty>::from_str(value.as_ref())
                        .map_err(|err| DecodeError::conversion(ctx.span(&scalar), err)),
                    Some(_) => Err(DecodeError::unexpected(
                        ctx.span(&scalar),
                        "radix",
                        "radix other than 10 (decimal) is not implemented",
                    )),
                    None => Err(DecodeError::scalar_kind(
                        ctx.span(&scalar),
                        "decimal",
                        scalar.literal.clone(),
                    )),
                }
                // <$ty>::from_str(scalar.literal.as_ref())
            }
        }

        impl EncodeScalar for $ty {
            fn encode(&self, _: &mut Context) -> Result<Scalar, EncodeError> {
                let literal = format!("{}", self);
                Ok(Scalar {
                    type_name: None,
                    literal: literal.into(),
                })
            }
        }
    };
}

impl_decimal!(f32);
impl_decimal!(f64);

impl DecodeScalar for String {
    fn decode(scalar: &Scalar, ctx: &mut Context) -> Result<Self, DecodeError> {
        if let Some(typ) = scalar.type_name.as_ref() {
            return Err(DecodeError::TypeName {
                span: ctx.span(&typ),
                found: Some(typ.clone()),
                expected: ExpectedType::no_type(),
                rust_type: "String",
            });
        }
        Ok(scalar.literal.clone().into())
    }
}
impl EncodeScalar for String {
    fn encode(&self, _: &mut Context) -> Result<Scalar, EncodeError> {
        let literal = format!("{:?}", self);
        Ok(Scalar {
            type_name: None,
            literal: literal.into(),
        })
    }
}

macro_rules! impl_from_str {
    ($ty:ty) => {
        impl DecodeScalar for $ty {
            fn decode(scalar: &crate::ast::Scalar, ctx: &mut Context) -> Result<Self, DecodeError> {
                if let Some(typ) = scalar.type_name.as_ref() {
                    return Err(DecodeError::TypeName {
                        span: ctx.span(&typ),
                        found: Some(typ.clone()),
                        expected: ExpectedType::no_type(),
                        rust_type: stringify!($ty),
                    });
                }
                <$ty>::from_str(scalar.literal.as_ref())
                    .map_err(|err| DecodeError::conversion(ctx.span(&scalar), err))
            }
        }
    };
}

#[cfg(feature = "std")]
mod _std {
    extern crate std;
    use super::*;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    impl_from_str!(PathBuf);
    impl EncodeScalar for PathBuf {
        fn encode(&self, _: &mut Context) -> Result<Scalar, EncodeError> {
            let string = format!("{}", self.display());
            Ok(Scalar {
                type_name: None,
                literal: string.into_boxed_str(),
            })
        }
    }

    impl_from_str!(SocketAddr);
    impl EncodeScalar for SocketAddr {
        fn encode(&self, _: &mut Context) -> Result<Scalar, EncodeError> {
            let string = format!("{}", self);
            Ok(Scalar {
                type_name: None,
                literal: string.into_boxed_str(),
            })
        }
    }
}

#[cfg(feature = "chrono")]
mod _chrono {
    use super::*;
    use chrono::NaiveDateTime;
    impl_from_str!(NaiveDateTime);
    impl EncodeScalar for NaiveDateTime {
        fn encode(&self, _: &mut Context) -> Result<Scalar, EncodeError> {
            let string = format!("{}", self);
            Ok(Scalar {
                type_name: None,
                literal: string.into_boxed_str(),
            })
        }
    }
}

#[cfg(feature = "http")]
mod _http {
    use super::*;
    use http::Uri;
    impl_from_str!(Uri);
    impl EncodeScalar for Uri {
        fn encode(&self, _: &mut Context) -> Result<Scalar, EncodeError> {
            let string = format!("{}", self);
            Ok(Scalar {
                type_name: None,
                literal: string.into_boxed_str(),
            })
        }
    }
}

impl DecodeScalar for bool {
    fn decode(scalar: &Scalar, ctx: &mut Context) -> Result<Self, DecodeError> {
        if let Some(typ) = scalar.type_name.as_ref() {
            return Err(DecodeError::TypeName {
                span: ctx.span(&typ),
                found: Some(typ.clone()),
                expected: ExpectedType::no_type(),
                rust_type: "bool",
            });
        }
        match scalar.literal.as_ref() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(DecodeError::scalar_kind(
                ctx.span(&scalar),
                "boolean",
                scalar.literal.clone(),
            )),
        }
    }
}
impl EncodeScalar for bool {
    fn encode(&self, _: &mut Context) -> Result<Scalar, EncodeError> {
        let literal = match self {
            true => "true",
            false => "false",
        };
        Ok(Scalar {
            type_name: None,
            literal: literal.into(),
        })
    }
}
