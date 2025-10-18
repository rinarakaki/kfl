mod common;

use kfl::Decode;
use std::path::PathBuf;

#[derive(Decode, Debug, PartialEq)]
struct Scalars {
    #[kfl(argument)]
    str: String,
    #[kfl(argument)]
    u64: u64,
    #[kfl(argument)]
    i641: i64,
    #[kfl(argument)]
    i64_plus: i64,
    #[kfl(argument)]
    i64_minus: i64,
    #[kfl(argument)]
    i64_b: i64,
    #[kfl(argument)]
    i64_o: i64,
    #[kfl(argument)]
    i64_x: i64,
    #[kfl(argument)]
    i64_b_minus: i64,
    #[kfl(argument)]
    i64_o_minus: i64,
    #[kfl(argument)]
    i64_x_minus: i64,
    #[kfl(argument)]
    f64: f64,
    #[kfl(argument)]
    path: PathBuf,
    #[kfl(argument)]
    boolean: bool,
}

#[test]
fn decode_types() {
    assert_decode!(
        r#"
        scalars \
            "hello" \
            1234 \
            1_234 \
            +1234 \
            -1234 \
            0b1 \
            0o7 \
            0xf \
            -0b1 \
            -0o7 \
            -0xf \
            1.234 \
            /hello/world \
            true \
    "#,
        Scalars {
            str: "hello".into(),
            u64: 1234,
            i641: 1234,
            i64_plus: 1234,
            i64_minus: -1234,
            i64_b: 1,
            i64_o: 7,
            i64_x: 15,
            i64_b_minus: -1,
            i64_o_minus: -7,
            i64_x_minus: -15,
            f64: 1.234,
            path: PathBuf::from("/hello/world"),
            boolean: true,
        }
    );
}
