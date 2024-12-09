//! This module provides a trait to convert a Rust type into a v8 Value.

/// The `IntoValue` trait is used to convert a Rust type into a v8 Value.
pub trait IntoValue {
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value>;
}

impl IntoValue for bool {
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        v8::Boolean::new(scope, self).into()
    }
}

impl IntoValue for i32 {
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        v8::Integer::new(scope, self).into()
    }
}

impl IntoValue for u32 {
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        v8::Integer::new_from_unsigned(scope, self).into()
    }
}

impl IntoValue for i64 {
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        v8::BigInt::new_from_i64(scope, self).into()
    }
}

impl IntoValue for f64 {
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        v8::Number::new(scope, self).into()
    }
}

impl IntoValue for f32 {
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        (self as f64).into_value(scope)
    }
}

impl IntoValue for String {
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        v8::String::new(scope, &self).unwrap_or(v8::String::empty(scope)).into()
    }
}

impl<T> IntoValue for Option<T>
where
    T: IntoValue,
{
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        match self {
            Some(value) => value.into_value(scope),
            None => v8::null(scope).into(),
        }
    }
}

impl<T> IntoValue for Vec<T>
where
    T: IntoValue,
{
    fn into_value<'a>(self, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        let l = i32::try_from(self.len()).unwrap_or(i32::MAX);
        let array = v8::Array::new(scope, l);

        for (i, value) in self.into_iter().enumerate() {
            let el: v8::Local<'_, v8::Value> = value.into_value(scope);
            array.set_index(scope, i as u32, el);
        }

        array.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::{into::IntoValue, setup};
    use v8::{ContextOptions, CreateParams};

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn can_convert_into_an_array() {
        setup::setup_test();
        let isolate = &mut v8::Isolate::new(CreateParams::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(scope, ContextOptions::default());
        let scope = &mut v8::ContextScope::new(scope, context);

        let array = vec![1, 2, 3, 4, 5];
        let array_value = array.into_value(scope);

        // cast the value to an array
        let array = array_value.try_cast::<v8::Array>().expect("Expected an array");
        assert_eq!(array.length(), 5);
        for i in 0..5 {
            let value = array.get_index(scope, i).unwrap();
            assert_eq!(value.to_int32(scope).unwrap().value(), i as i32 + 1);
        }
    }
}
