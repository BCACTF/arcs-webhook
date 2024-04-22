
use serde_json::{Map, Value};

pub trait JsonAccessors<'a> {
    fn null(self) -> Option<()>;
    fn bool(self) -> Option<bool>;

    fn float(self) -> Option<f64>;
    fn int(self) -> Option<u64>;
    fn str(self) -> Option<&'a str>;
    
    fn arr(self) -> Option<&'a [Value]>;
    fn obj(self) -> Option<&'a Map<String, Value>>;
}

impl<'a> JsonAccessors<'a> for &'a Value {
    fn null(self) -> Option<()> { Some(self).null() }
    fn bool(self) -> Option<bool> { Some(self).bool() }
    fn float(self) -> Option<f64> { Some(self).float() }
    fn int(self) -> Option<u64> { Some(self).int() }
    fn str(self) -> Option<&'a str> { Some(self).str() }
    fn arr(self) -> Option<&'a [Value]> { Some(self).arr() }
    fn obj(self) -> Option<&'a Map<String, Value>> { Some(self).obj() }
}
impl<'a> JsonAccessors<'a> for Option<&'a Value> {
    fn null(self) -> Option<()> { self?.as_null() }
    fn bool(self) -> Option<bool> { self?.as_bool() }
    fn float(self) -> Option<f64> { self?.as_f64() }
    fn int(self) -> Option<u64> { self?.as_u64() }
    fn str(self) -> Option<&'a str> { self?.as_str() }

    fn arr(self) -> Option<&'a [Value]> {
        self?.as_array().map(Vec::as_slice)
    }
    fn obj(self) -> Option<&'a Map<String, Value>> { self?.as_object() }
}

pub trait PseudoIndex<'a, T> {
    fn index(&self, index: T) -> Option<&'a Value>;
}


impl<'a> PseudoIndex<'a, usize> for &'a [Value] {
    fn index(&self, index: usize) -> Option<&'a Value> {
        Some(*self).index(index)
    }
}
impl<'a> PseudoIndex<'a, usize> for Option<&'a [Value]> {
    fn index(&self, index: usize) -> Option<&'a Value> {
        (*self)?.get(index)
    }
}

impl<'a> PseudoIndex<'a, &str> for &'a Map<String, Value> {
    fn index(&self, index: &str) -> Option<&'a Value> {
        Some(*self).index(index)
    }
}
impl<'a> PseudoIndex<'a, &str> for Option<&'a Map<String, Value>> {
    fn index(&self, index: &str) -> Option<&'a Value> {
        (*self)?.get(index)
    }
}
