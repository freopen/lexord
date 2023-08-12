use insta::assert_snapshot;

use lexord::util::test::encode;
use lexord::LexOrd;

#[test]
fn test_struct() {
    #[derive(LexOrd, Debug)]
    struct A {
        a: u16,
        b: u16,
    }

    #[derive(LexOrd, Debug)]
    struct B {
        a: u16,
        b: A,
        c: u16,
    }

    assert_snapshot!(encode(A { a: 1, b: 2 }), @"81 82");
    assert_snapshot!(encode(B { a: 1, b: A { a: 2, b: 3 }, c: 4 }), @"81 82 83 84");
}

#[test]
fn test_enum() {
    #[derive(LexOrd, Debug)]
    enum A {
        A,
        B(u16, u16),
        C { a: u16, b: u16 },
    }

    assert_snapshot!(encode(A::A), @"80");
    assert_snapshot!(encode(A::B(1, 2)), @"81 81 82");
    assert_snapshot!(encode(A::C { a: 1, b: 2 }), @"82 81 82");
}
