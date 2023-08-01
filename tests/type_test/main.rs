use std::cmp::Ordering;

use bolero::{check, TypeGenerator};
use lexord::{LexOrd, LexOrdSer};

#[derive(LexOrd, Debug, TypeGenerator)]
struct Test {
    t_u8: u8,
    t_u16: u16,
    t_u32: u32,
    t_u64: u64,
    t_u128: u128,
    t_usize: usize,
    t_i8: i8,
    t_i16: i16,
    t_i32: i32,
    t_i64: i64,
    t_i128: i128,
    t_isize: isize,
    t_f32: f32,
    t_f64: f64,
    t_string: String,
}

fn main() {
    check!().with_type::<(Test, Test)>().for_each(|data| {
        let mut ser_0 = vec![];
        data.0.to_write(&mut ser_0).unwrap();
        let mut ser_1 = vec![];
        data.1.to_write(&mut ser_1).unwrap();
        match data.0.partial_cmp(&data.1) {
            None | Some(Ordering::Equal) => {}
            Some(ordering) => {
                assert_eq!(ordering, ser_0.cmp(&ser_1), "{:?} vs {:?}", data.0, data.1);
            }
        }
        match data
            .0
            .partial_cmp(&Test::from_read(&mut ser_0.as_slice()).unwrap())
        {
            None | Some(Ordering::Equal) => {}
            _ => panic!("{:?}", data.0),
        }
        match data
            .1
            .partial_cmp(&Test::from_read(&mut ser_1.as_slice()).unwrap())
        {
            None | Some(Ordering::Equal) => {}
            _ => panic!("{:?}", data.1),
        }
    });
}
