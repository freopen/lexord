use lexord_fuzz_macros::define_anyvalue;
use quote::ToTokens;
use std::{
    cell::Cell,
    fmt::{Debug, Display},
    mem::take,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct AnyType {
    type_id: u16,
    children: Vec<AnyType>,
}
thread_local! {
    static CURRENT_TYPE: Cell<Vec<AnyType>> = const { Cell::new(vec![]) }
}

const MAX_TYPE_SIZE: usize = 10;

impl AnyType {
    fn size(&self) -> usize {
        1 + self.children.iter().map(|c| c.size()).sum::<usize>()
    }
    pub fn random(data: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        let type_id: u16 = data.arbitrary()?;
        if type_id >= NUM_PARAMS.len() as u16 {
            return Err(arbitrary::Error::IncorrectFormat);
        }
        let children = (0..NUM_PARAMS[type_id as usize])
            .map(|_| AnyType::random(data))
            .collect::<arbitrary::Result<Vec<_>>>()?;
        let output = AnyType { type_id, children };
        if output.size() > MAX_TYPE_SIZE {
            return Err(arbitrary::Error::IncorrectFormat);
        }
        Ok(output)
    }
    pub fn set_current_type(self) {
        CURRENT_TYPE.set(vec![self]);
    }
}

impl Display for AnyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_syn().to_token_stream())
    }
}

struct InferToType<'a>(usize, &'a [AnyType]);

impl<'a> syn::fold::Fold for InferToType<'a> {
    fn fold_type(&mut self, ty: syn::Type) -> syn::Type {
        match ty {
            syn::Type::Infer(_) => {
                let i = self.0;
                self.0 += 1;
                self.1[i].as_syn()
            }
            _ => syn::fold::fold_type(self, ty),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct AnyValue<const CHILD_INDEX: usize>(Box<AnyValueEnum>);

impl<const CHILD_INDEX: usize> AnyValue<CHILD_INDEX> {
    fn with<F: FnOnce(u16) -> R, R>(f: F) -> R {
        let mut outer = CURRENT_TYPE.take();
        CURRENT_TYPE.set(take(&mut outer[CHILD_INDEX].children));
        let r = f(outer[CHILD_INDEX].type_id);
        outer[CHILD_INDEX].children = CURRENT_TYPE.take();
        CURRENT_TYPE.set(outer);
        r
    }
}

define_anyvalue!(
    (),
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    String,
    (_,),
    (_, _),
    (_, _, _),
);
