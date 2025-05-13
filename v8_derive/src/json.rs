use crate::errors::{Error, Result};
use crate::TryFromValue;
use v8::{HandleScope, Local, Value};

/// Convert a V8 Object to a JSON Value
/// # Errors
/// In case of conversion errors
pub(crate) fn v8_to_json_value(scope: &mut HandleScope, value: Local<Value>) -> Result<serde_json::Value> {
    if value.is_string() {
        let value = String::try_from_value(&value, scope)?;
        Ok(serde_json::Value::String(value))
    } else if value.is_int32() {
        let value = i32::try_from_value(&value, scope)?;
        Ok(serde_json::Value::from(value))
    } else if value.is_uint32() {
        let value = u32::try_from_value(&value, scope)?;
        Ok(serde_json::Value::from(value))
    } else if value.is_big_int() {
        let value = i64::try_from_value(&value, scope)?;
        Ok(serde_json::Value::from(value))
    } else if value.is_number() {
        let value = f64::try_from_value(&value, scope)?;
        Ok(serde_json::Value::from(value))
    } else if value.is_boolean() {
        let value = bool::try_from_value(&value, scope)?;
        Ok(serde_json::Value::from(value))
    } else if value.is_null() {
        Ok(serde_json::Value::Null)
    } else if value.is_array() {
        let Ok(array) = value.try_cast::<v8::Array>() else {
            return Err(Error::ExpectedArray);
        };
        let length = array.length();
        let mut json_array = Vec::with_capacity(length as usize);
        for i in 0..length {
            let item = match array.get_index(scope, i) {
                Some(item) => v8_to_json_value(scope, item)?,
                None => serde_json::Value::Null,
            };
            json_array.push(item);
        }
        Ok(json_array.into())
    } else if value.is_object() {
        let Some(object) = value.to_object(scope) else {
            return Err(Error::ExpectedObject);
        };
        let Some(properties) = object.get_property_names(scope, v8::GetPropertyNamesArgs::default()) else {
            return Err(Error::FailedToGetPropertyNames);
        };
        let length = properties.length();
        let mut json_object = serde_json::Map::new();
        for i in 0..length {
            let Some(key) = properties.get_index(scope, i) else {
                return Err(Error::ExpectedObject);
            };
            let key_str = String::try_from_value(&key, scope)?;
            let Some(value) = object.get(scope, key) else {
                return Err(Error::ExpectedObject);
            };
            let value = v8_to_json_value(scope, value)?;
            json_object.insert(key_str, value);
        }
        Ok(serde_json::Value::Object(json_object))
    } else {
        Err(Error::UnsupportedValueType)
    }
}
