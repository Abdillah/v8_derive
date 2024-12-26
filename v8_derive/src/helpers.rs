use crate::{errors, from::TryFromValue};
use std::{collections::HashMap, hash::BuildHasher};
use v8::GetPropertyNamesArgs;

pub fn get_field_as<'a, T>(
    field_name: &str,
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
    parse_fn: ParseFn<T>,
) -> errors::Result<T> {
    if !input.is_object() {
        return Err(errors::Error::ExpectedObject);
    };

    let js_object: v8::Local<v8::Object> = input.try_cast()?;
    let js_key = v8::String::new(scope, field_name)
        .map(Into::into)
        .ok_or(errors::Error::InvalidField(field_name.to_string()))?;
    let js_value = js_object
        .get(scope, js_key)
        .ok_or(errors::Error::FieldNoFound(field_name.to_string()))?;

    parse_fn(&js_value, scope)
}

pub fn get_optional_field_as<'a, T>(
    field_name: &str,
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
    parse_fn: ParseFn<T>,
) -> errors::Result<Option<T>> {
    if !input.is_object() {
        return Err(errors::Error::ExpectedObject);
    };

    let js_object: v8::Local<v8::Object> = input.try_cast()?;
    let js_key = v8::String::new(scope, field_name)
        .map(Into::into)
        .ok_or(errors::Error::InvalidField(field_name.to_string()))?;
    let js_value = js_object.get(scope, js_key);

    // field not found
    let Some(js_value) = js_value else {
        return Ok(None);
    };

    // check for null
    if js_value.is_null_or_undefined() {
        return Ok(None);
    }

    let inner_value = parse_fn(&js_value, scope)?;
    Ok(Some(inner_value))
}

pub type ParseFn<T> = fn(&'_ v8::Local<'_, v8::Value>, &'_ mut v8::HandleScope<'_>) -> errors::Result<T>;

pub fn try_as_bool<'a>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<bool> {
    if !input.is_boolean() {
        return Err(errors::Error::ExpectedBoolean);
    };

    Ok(input.boolean_value(scope))
}

pub fn try_as_string<'a>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<String> {
    // try to convert the value to String anyway
    // if !input.is_string() {
    //     return Err(errors::Error::ExpectedString);
    // };
    Ok(input.to_rust_string_lossy(scope))
}

pub fn try_as_i32<'a>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<i32> {
    if !input.is_int32() {
        return Err(errors::Error::ExpectedI32);
    };

    input.int32_value(scope).ok_or(errors::Error::ExpectedI32)
}

pub fn try_as_u32<'a>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<u32> {
    if !input.is_uint32() {
        return Err(errors::Error::ExpectedI32);
    };

    input.uint32_value(scope).ok_or(errors::Error::ExpectedI32)
}

pub fn try_as_i64<'a>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<i64> {
    if !input.is_big_int() {
        return Err(errors::Error::ExpectedI64);
    };

    let i = input.to_big_int(scope).ok_or(errors::Error::ExpectedI64)?;
    Ok(i.i64_value().0)
}

pub fn try_as_f64<'a>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<f64> {
    if !input.is_number() {
        return Err(errors::Error::ExpectedF64);
    };

    input.number_value(scope).ok_or(errors::Error::ExpectedF64)
}

pub fn try_as_f32<'a>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<f32> {
    let i = try_as_f64(input, scope)?;
    Ok(i as f32)
}

pub fn try_as_i8<'a>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<i8> {
    let i = try_as_i32(input, scope)?;
    i8::try_from(i).map_err(|_| errors::Error::OutOfRange)
}

pub fn try_as_vec<'a, T>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<Vec<T>>
where
    T: TryFromValue,
{
    if !input.is_array() {
        return Err(errors::Error::ExpectedArray);
    };

    let array: v8::Local<v8::Array> = input.try_cast()?;
    let length = array.length();

    let mut result = Vec::with_capacity(length as usize);

    for i in 0..length {
        let Some(element) = array.get_index(scope, i) else {
            // this should never happen
            continue;
        };

        let element = T::try_from_value(&element, scope)?;
        result.push(element);
    }

    Ok(result)
}

pub fn try_as_hashmap<'a, T, S>(
    input: &'a v8::Local<'a, v8::Value>,
    scope: &'a mut v8::HandleScope<'_, v8::Context>,
) -> errors::Result<HashMap<String, T, S>>
where
    T: TryFromValue,
    S: BuildHasher + Default,
{
    println!("Input is_map={}, is_object={}", input.is_map(), input.is_object());
    if !(input.is_map() || input.is_object()) {
        return Err(errors::Error::ExpectedMap);
    };

    let mut result: HashMap<String, T, S> = HashMap::with_hasher(S::default());

    if input.is_map() {
        let js_map: v8::Local<v8::Map> = input.try_cast()?;
        let js_array = js_map.as_array(scope); // js_array is twice the size of the map; odd indexes are keys, even indexes are values
        for i in (0..js_array.length()).step_by(2) {
            let (Some(key), Some(value)) = (js_array.get_index(scope, i), js_array.get_index(scope, i + 1)) else {
                continue;
            };

            let key = key.to_rust_string_lossy(scope);
            let value = T::try_from_value(&value, scope)?;
            result.insert(key, value);
        }

        return Ok(result);
    }

    // object
    let js_object: v8::Local<v8::Object> = input.try_cast()?;
    let keys = js_object
        .get_own_property_names(scope, GetPropertyNamesArgs::default())
        .ok_or(errors::Error::FailedToGetPropertyNames)?;

    for i in 0..keys.length() {
        let key = keys
            .get_index(scope, i)
            .ok_or(errors::Error::FailedToGetPropertyNames)?;
        let value = js_object
            .get(scope, key)
            .ok_or(errors::Error::FailedToGetPropertyNames)?;
        let value = T::try_from_value(&value, scope)?;
        let key = key.to_rust_string_lossy(scope);
        result.insert(key, value);
    }

    Ok(result)
}

#[cfg(test)]
pub(crate) mod setup {
    use std::sync::Once;

    /// Set up global state for a test
    pub(crate) fn setup_test() {
        initialize_once();
    }

    fn initialize_once() {
        static START: Once = Once::new();
        START.call_once(|| {
            v8::V8::set_flags_from_string(
                "--no_freeze_flags_after_init --expose_gc --harmony-import-assertions --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls",
            );
            v8::V8::initialize_platform(
                v8::new_unprotected_default_platform(0, false).make_shared(),
            );
            v8::V8::initialize();
        });
    }
}
