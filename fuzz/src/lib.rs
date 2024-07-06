use lexord::LexOrd;
use lexord_fuzz_macros::define_anyvalue;
use quote::ToTokens;
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    mem::take,
};

#[derive(Debug, Clone, LexOrd, Eq, Ord)]
pub struct AnyType {
    pub type_id: u16,
    pub children: Vec<AnyType>,
}
thread_local! {
    static CURRENT_TYPE: RefCell<Vec<AnyType>> = Default::default();
}

#[cfg(feature = "golden")]
thread_local! {
    pub static TYPE_TO_UNSTRUCTURED: std::cell::RefCell<std::collections::BTreeMap<AnyType,Vec<Vec<u8>>>> = Default::default();
}
#[cfg(feature = "golden")]
fn log_for_golden(data: &arbitrary::Unstructured, child_index: usize) {
    let any_type = CURRENT_TYPE.with_borrow(|t| t[child_index].clone());
    let unstructured = data.peek_bytes(data.len()).unwrap().to_vec();
    TYPE_TO_UNSTRUCTURED.with_borrow_mut(|map| {
        map.entry(any_type).or_default().push(unstructured);
    })
}
#[cfg(not(feature = "golden"))]
fn log_for_golden(_data: &arbitrary::Unstructured, _child_index: usize) {}

impl AnyType {
    pub fn random(data: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        const MAX_SIZE: usize = 5;
        let mut size_left = MAX_SIZE;
        Self::random_recursive(data, &mut size_left)
    }
    pub fn random_recursive(
        data: &mut arbitrary::Unstructured,
        size_left: &mut usize,
    ) -> arbitrary::Result<Self> {
        if *size_left == 0 {
            return Err(arbitrary::Error::IncorrectFormat);
        }
        *size_left -= 1;
        let type_id: u16 = data.arbitrary()?;
        if type_id >= NUM_PARAMS.len() as u16 {
            return Err(arbitrary::Error::IncorrectFormat);
        }
        let children = (0..NUM_PARAMS[type_id as usize])
            .map(|_| AnyType::random_recursive(data, size_left))
            .collect::<arbitrary::Result<Vec<_>>>()?;
        let output = AnyType { type_id, children };
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
    Vec<_>,
);
