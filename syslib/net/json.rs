use alloc::boxed::Box;
/*
 * Nuva OS - SystemLibrary - Net
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.

 */

// ! JSON encodingDecode

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

/// JSON valueType
#[derive(Debug, Clone)]
pub enum JsonValue {
 Null,
 Bool(bool),
 Number(f64),
 String([u8; 1024], u16),
 Array([Box<JsonValue>; 64], u8),
 Object([( [u8; 64], u8, Box<JsonValue>); 32], u8),
}

impl JsonValue {
 pub fn null() -> Self {
 JsonValue::Null
 }

 pub fn bool(v: bool) -> Self {
 JsonValue::Bool(v)
 }

 pub fn number(v: f64) -> Self {
 JsonValue::Number(v)
 }

 pub fn string(s: &[u8]) -> Self {
 let mut buf = [0u8; 1024];
 let len = s.len().min(1023);
 buf[..len].copy_from_slice(&s[..len]);
 JsonValue::String(buf, len as u16)
 }

 pub fn array() -> Self {
     // TODO: Box<JsonValue> array initialization
     let arr: [Box<JsonValue>; 64] = core::array::from_fn(|_| Box::new(JsonValue::Null));
     JsonValue::Array(arr, 0)
 }

 pub fn object() -> Self {
     // TODO: Box<JsonValue> object initialization
     let obj: [([u8; 64], u8, Box<JsonValue>); 32] = core::array::from_fn(|_| ([0u8; 64], 0, Box::new(JsonValue::Null)));
     JsonValue::Object(obj, 0)
 }

 pub fn is_null(&self) -> bool {
 matches!(self, JsonValue::Null)
 }

 pub fn is_bool(&self) -> bool {
 matches!(self, JsonValue::Bool(_))
 }

 pub fn is_number(&self) -> bool {
 matches!(self, JsonValue::Number(_))
 }

 pub fn is_string(&self) -> bool {
 matches!(self, JsonValue::String(_, _))
 }

 pub fn is_array(&self) -> bool {
 matches!(self, JsonValue::Array(_, _))
 }

 pub fn is_object(&self) -> bool {
 matches!(self, JsonValue::Object(_, _))
 }

 pub fn as_bool(&self) -> Option<bool> {
 match self {
 JsonValue::Bool(v) => Some(*v),
 _ => None,
 }
 }

 pub fn as_number(&self) -> Option<f64> {
 match self {
 JsonValue::Number(v) => Some(*v),
 _ => None,
 }
 }

 pub fn as_str(&self) -> Option<&[u8]> {
 match self {
 JsonValue::String(buf, len) => Some(&buf[..*len as usize]),
 _ => None,
 }
 }

 pub fn push(&mut self, value: JsonValue) -> bool {
 match self {
 JsonValue::Array(ref mut arr, ref mut len) => {
 if *len < 64 {
 arr[*len as usize] = Box::new(value);
 *len += 1;
 return true;
 }
 }
 _ => {}
 }
 false
 }

 pub fn insert(&mut self, key: &[u8], value: JsonValue) -> bool {
 match self {
 JsonValue::Object(ref mut obj, ref mut len) => {
 if *len < 32 {
 let mut key_buf = [0u8; 64];
 let key_len = key.len().min(63);
 key_buf[..key_len].copy_from_slice(&key[..key_len]);
 obj[*len as usize] = (key_buf, key_len as u8, Box::new(value));
 *len += 1;
 return true;
 }
 }
 _ => {}
 }
 false
 }

 pub fn get(&self, key: &[u8]) -> Option<&JsonValue> {
 match self {
 JsonValue::Object(obj, len) => {
 for i in 0..*len as usize {
 let (k, klen, v) = &obj[i];
 if &k[..*klen as usize] == key {
 return Some(v);
 }
 }
 None
 }
 _ => None,
 }
 }

 pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
 match self {
 JsonValue::Array(arr, len) => {
 if index < *len as usize {
 Some(&arr[index])
 } else {
 None
 }
 }
 _ => None,
 }
 }
}

/// JSON parsedevice
pub struct JsonParser {
 data: [u8; 65536],
 len: usize,
 pos: usize,
}

impl JsonParser {
 pub fn new() -> Self {
 Self {
 data: [0; 65536],
 len: 0,
 pos: 0,
 }
 }

 pub fn parse(&mut self, json: &[u8]) -> Result<JsonValue, JsonError> {
 let len = json.len().min(65535);
 self.data[..len].copy_from_slice(&json[..len]);
 self.len = len;
 self.pos = 0;
 
 self.skip_whitespace();
 self.parse_value()
 }

 fn parse_value(&mut self) -> Result<JsonValue, JsonError> {
 if self.pos >= self.len {
 return Err(JsonError::UnexpectedEnd);
 }
 
 let c = self.data[self.pos];
 
 match c {
 b'n' => self.parse_null(),
 b't' | b'f' => self.parse_bool(),
 b'"' => self.parse_string(),
 b'[' => self.parse_array(),
 b'{' => self.parse_object(),
 b'-' | b'0'..=b'9' => self.parse_number(),
 _ => Err(JsonError::UnexpectedToken),
 }
 }

 fn parse_null(&mut self) -> Result<JsonValue, JsonError> {
 if self.pos + 4 <= self.len && &self.data[self.pos..self.pos + 4] == b"null" {
 self.pos += 4;
 return Ok(JsonValue::null());
 }
 Err(JsonError::InvalidLiteral)
 }

 fn parse_bool(&mut self) -> Result<JsonValue, JsonError> {
 if self.pos + 4 <= self.len && &self.data[self.pos..self.pos + 4] == b"true" {
 self.pos += 4;
 return Ok(JsonValue::Bool(true));
 }
 if self.pos + 5 <= self.len && &self.data[self.pos..self.pos + 5] == b"false" {
 self.pos += 5;
 return Ok(JsonValue::Bool(false));
 }
 Err(JsonError::InvalidLiteral)
 }

 fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
 let start = self.pos;
 
 // signal
 if self.pos < self.len && self.data[self.pos] == b'-' {
 self.pos += 1;
 }
 
 // Integerpartsplit
 while self.pos < self.len && self.data[self.pos].is_ascii_digit() {
 self.pos += 1;
 }
 
 // smallnumberpartsplit
 if self.pos < self.len && self.data[self.pos] == b'.' {
 self.pos += 1;
 while self.pos < self.len && self.data[self.pos].is_ascii_digit() {
 self.pos += 1;
 }
 }
 
 // Exponentialpartsplit
 if self.pos < self.len && (self.data[self.pos] == b'e' || self.data[self.pos] == b'E') {
 self.pos += 1;
 if self.pos < self.len && (self.data[self.pos] == b'+' || self.data[self.pos] == b'-') {
 self.pos += 1;
 }
 while self.pos < self.len && self.data[self.pos].is_ascii_digit() {
 self.pos += 1;
 }
 }
 
 let num_str = &self.data[start..self.pos];
 let mut value = 0.0f64;
 let mut decimal = false;
 let mut divisor = 1.0f64;
 let mut negative = false;
 let mut idx = 0;
 
 if idx < num_str.len() && num_str[idx] == b'-' {
 negative = true;
 idx += 1;
 }
 
 while idx < num_str.len() {
 let c = num_str[idx];
 if c == b'.' {
 decimal = true;
 } else if c.is_ascii_digit() {
 let digit = (c - b'0') as f64;
 if decimal {
 divisor *= 10.0;
 value += digit / divisor;
 } else {
 value = value * 10.0 + digit;
 }
 } else if c == b'e' || c == b'E' {
 // Simplified: tacticExponential
 break;
 }
 idx += 1;
 }
 
 if negative {
 value = -value;
 }
 
 Ok(JsonValue::Number(value))
 }

 fn parse_string(&mut self) -> Result<JsonValue, JsonError> {
 self.pos += 1; // jumpoveropenHead "
 
 let mut buf = [0u8; 1024];
 let mut len = 0;
 
 while self.pos < self.len && self.data[self.pos] != b'"' && len < 1023 {
 if self.data[self.pos] == b'\\' {
 self.pos += 1;
 if self.pos < self.len {
 let escaped = match self.data[self.pos] {
 b'n' => b'\n',
 b't' => b'\t',
 b'r' => b'\r',
 b'\\' => b'\\',
 b'"' => b'"',
 b'/' => b'/',
 b'u' => {
 // Unicode branchmeaning, SimplifiedHandle
 self.pos += 4;
 b'?'
 }
 c => c,
 };
 buf[len] = escaped;
 len += 1;
 }
 } else {
 buf[len] = self.data[self.pos];
 len += 1;
 }
 self.pos += 1;
 }
 
 if self.pos < self.len {
 self.pos += 1; // jumpoverTail "
 }
 
 Ok(JsonValue::String(buf, len as u16))
 }

 fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
 self.pos += 1; // jumpover [
 self.skip_whitespace();
 
 let mut arr: [Box<JsonValue>; 64] = core::array::from_fn(|_| Box::new(JsonValue::Null));
 let mut len = 0;
 
 if self.pos < self.len && self.data[self.pos] != b']' {
 loop {
 if len >= 64 {
 return Err(JsonError::ArrayTooLong);
 }
 
 arr[len] = Box::new(self.parse_value()?);
 len += 1;
 
 self.skip_whitespace();
 
 if self.pos >= self.len {
 return Err(JsonError::UnexpectedEnd);
 }
 
 if self.data[self.pos] == b']' {
 break;
 }
 
 if self.data[self.pos] != b',' {
 return Err(JsonError::ExpectedComma);
 }
 self.pos += 1;
 self.skip_whitespace();
 }
 }
 
 self.pos += 1; // jumpover ]
 Ok(JsonValue::Array(arr, len as u8))
 }

 fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
 self.pos += 1; // jumpover {
 self.skip_whitespace();
 
 let mut obj: [([u8; 64], u8, Box<JsonValue>); 32] = core::array::from_fn(|_| ([0u8; 64], 0u8, Box::new(JsonValue::Null)));
 let mut len = 0;
 
 if self.pos < self.len && self.data[self.pos] != b'}' {
 loop {
 if len >= 32 {
 return Err(JsonError::ObjectTooLong);
 }
 
 // parse key
 self.skip_whitespace();
 if self.pos >= self.len || self.data[self.pos] != b'"' {
 return Err(JsonError::ExpectedString);
 }
 
 let key_value = self.parse_string()?;
 let key = match key_value {
 JsonValue::String(buf, klen) => {
 let mut key_buf = [0u8; 64];
 let key_len = klen as usize;
 key_buf[..key_len].copy_from_slice(&buf[..key_len]);
 (key_buf, klen as u8)
 }
 _ => return Err(JsonError::ExpectedString),
 };
 
 self.skip_whitespace();
 
 if self.pos >= self.len || self.data[self.pos] != b':' {
 return Err(JsonError::ExpectedColon);
 }
 self.pos += 1;
 
 self.skip_whitespace();
 
 // parse value
 let value = self.parse_value()?;
 
 obj[len] = (key.0, key.1, Box::new(value));
 len += 1;
 
 self.skip_whitespace();
 
 if self.pos >= self.len {
 return Err(JsonError::UnexpectedEnd);
 }
 
 if self.data[self.pos] == b'}' {
 break;
 }
 
 if self.data[self.pos] != b',' {
 return Err(JsonError::ExpectedComma);
 }
 self.pos += 1;
 }
 }
 
 self.pos += 1; // jumpover }
 Ok(JsonValue::Object(obj, len as u8))
 }

 fn skip_whitespace(&mut self) {
 while self.pos < self.len {
 match self.data[self.pos] {
 b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
 _ => break,
 }
 }
 }
}

/// JSON Error
#[derive(Debug, Clone, Copy)]
pub enum JsonError {
 UnexpectedEnd,
 UnexpectedToken,
 InvalidLiteral,
 ExpectedString,
 ExpectedColon,
 ExpectedComma,
 ArrayTooLong,
 ObjectTooLong,
}

/// JSON Serializationdevice
pub struct JsonSerializer;

impl JsonSerializer {
 pub fn serialize(value: &JsonValue) -> Vec<u8> {
 let mut result = Vec::new();
 Self::serialize_value(value, &mut result);
 result
 }

 fn serialize_value(value: &JsonValue, output: &mut Vec<u8>) {
 match value {
 JsonValue::Null => {
 output.extend_from_slice(b"null");
 }
 JsonValue::Bool(v) => {
 if *v {
 output.extend_from_slice(b"true");
 } else {
 output.extend_from_slice(b"false");
 }
 }
 JsonValue::Number(v) => {
 // Simplified: IntegerFormat
 let int_val = *v as i64;
 let mut buf = [0u8; 32];
 let mut len = 0;
 let mut n = int_val.abs();
 
 if n == 0 {
 buf[0] = b'0';
 len = 1;
 } else {
 while n > 0 {
 buf[31 - len] = b'0' + (n % 10) as u8;
 n /= 10;
 len += 1;
 }
 }
 
 if int_val < 0 {
 output.push(b'-');
 }
 output.extend_from_slice(&buf[32 - len..]);
 }
 JsonValue::String(buf, len) => {
 output.push(b'"');
 for &b in &buf[..*len as usize] {
 match b {
 b'"' => output.extend_from_slice(b"\\\""),
 b'\\' => output.extend_from_slice(b"\\\\"),
 b'\n' => output.extend_from_slice(b"\\n"),
 b'\r' => output.extend_from_slice(b"\\r"),
 b'\t' => output.extend_from_slice(b"\\t"),
 c => output.push(c),
 }
 }
 output.push(b'"');
 }
 JsonValue::Array(arr, len) => {
 output.push(b'[');
 for i in 0..*len as usize {
 if i > 0 {
 output.push(b',');
 }
 Self::serialize_value(&arr[i], output);
 }
 output.push(b']');
 }
 JsonValue::Object(obj, len) => {
 output.push(b'{');
 for i in 0..*len as usize {
 if i > 0 {
 output.push(b',');
 }
 let (key, key_len, val) = &obj[i];
 output.push(b'"');
 output.extend_from_slice(&key[..*key_len as usize]);
 output.extend_from_slice(b"\":");
 Self::serialize_value(val, output);
 }
 output.push(b'}');
 }
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_json_value_null() {
 let value = JsonValue::null();

 assert!(value.is_null());
 assert!(!value.is_bool());
 assert!(!value.is_number());
 }

 #[test]
 fn test_json_value_bool() {
 let t = JsonValue::bool(true);
 let f = JsonValue::bool(false);

 assert!(t.is_bool());
 assert_eq!(t.as_bool(), Some(true));
 assert_eq!(f.as_bool(), Some(false));
 }

 #[test]
 fn test_json_value_number() {
 let value = JsonValue::number(3.14);

 assert!(value.is_number());
 assert_eq!(value.as_number(), Some(3.14));
 }

 #[test]
 fn test_json_value_string() {
 let value = JsonValue::string(b"hello");

 assert!(value.is_string());
 assert_eq!(value.as_str(), Some(&b"hello"[..]));
 }

 #[test]
 fn test_json_value_array() {
 let mut value = JsonValue::array();

 assert!(value.is_array());

 value.push(JsonValue::number(1.0));
 value.push(JsonValue::number(2.0));

 assert_eq!(value.get_index(0).unwrap().as_number(), Some(1.0));
 assert_eq!(value.get_index(1).unwrap().as_number(), Some(2.0));
 }

 #[test]
 fn test_json_value_object() {
 let mut value = JsonValue::object();

 assert!(value.is_object());

 value.insert(b"name", JsonValue::string(b"test"));
 value.insert(b"value", JsonValue::number(42.0));

 assert_eq!(value.get(b"name").unwrap().as_str(), Some(&b"test"[..]));
 assert_eq!(value.get(b"value").unwrap().as_number(), Some(42.0));
 }

 #[test]
 fn test_json_parser_null() {
 let mut parser = JsonParser::new();

 let result = parser.parse(b"null");
 assert!(result.is_ok());
 assert!(result.unwrap().is_null());
 }

 #[test]
 fn test_json_parser_bool() {
 let mut parser = JsonParser::new();

 let result = parser.parse(b"true");
 assert!(result.is_ok());
 assert_eq!(result.unwrap().as_bool(), Some(true));

 let result = parser.parse(b"false");
 assert!(result.is_ok());
 assert_eq!(result.unwrap().as_bool(), Some(false));
 }

 #[test]
 fn test_json_parser_number() {
 let mut parser = JsonParser::new();

 let result = parser.parse(b"42");
 assert!(result.is_ok());
 assert_eq!(result.unwrap().as_number(), Some(42.0));

 let result = parser.parse(b"-123");
 assert!(result.is_ok());
 assert_eq!(result.unwrap().as_number(), Some(-123.0));

 let result = parser.parse(b"3.14");
 assert!(result.is_ok());
 let num = result.unwrap().as_number().unwrap();
 assert!(num > 3.13 && num < 3.15);
 }

 #[test]
 fn test_json_parser_string() {
 let mut parser = JsonParser::new();

 let result = parser.parse(b"\"hello\"");
 assert!(result.is_ok());
 assert_eq!(result.unwrap().as_str(), Some(&b"hello"[..]));
 }

 #[test]
 fn test_json_parser_array() {
 let mut parser = JsonParser::new();

 let result = parser.parse(b"[1, 2, 3]");
 assert!(result.is_ok());

 let arr = result.unwrap();
 assert!(arr.is_array());
 assert_eq!(arr.get_index(0).unwrap().as_number(), Some(1.0));
 assert_eq!(arr.get_index(1).unwrap().as_number(), Some(2.0));
 assert_eq!(arr.get_index(2).unwrap().as_number(), Some(3.0));
 }

 #[test]
 fn test_json_parser_object() {
 let mut parser = JsonParser::new();

 let result = parser.parse(b"{\"name\": \"test\", \"value\": 42}");
 assert!(result.is_ok());

 let obj = result.unwrap();
 assert!(obj.is_object());
 assert_eq!(obj.get(b"name").unwrap().as_str(), Some(&b"test"[..]));
 assert_eq!(obj.get(b"value").unwrap().as_number(), Some(42.0));
 }

 #[test]
 fn test_json_parser_nested() {
 let mut parser = JsonParser::new();

 let result = parser.parse(b"{\"arr\": [1, 2], \"obj\": {\"x\": 10}}");
 assert!(result.is_ok());

 let obj = result.unwrap();
 let arr = obj.get(b"arr").unwrap();
 assert!(arr.is_array());

 let inner_obj = obj.get(b"obj").unwrap();
 assert!(inner_obj.is_object());
 }

 #[test]
 fn test_json_serializer_null() {
 let value = JsonValue::null();
 let result = JsonSerializer::serialize(&value);

 assert_eq!(&result[..], b"null");
 }

 #[test]
 fn test_json_serializer_bool() {
 let t = JsonValue::bool(true);
 let f = JsonValue::bool(false);

 assert_eq!(&JsonSerializer::serialize(&t)[..], b"true");
 assert_eq!(&JsonSerializer::serialize(&f)[..], b"false");
 }

 #[test]
 fn test_json_serializer_number() {
 let value = JsonValue::number(42.0);
 let result = JsonSerializer::serialize(&value);

 assert_eq!(&result[..], b"42");
 }

 #[test]
 fn test_json_serializer_string() {
 let value = JsonValue::string(b"hello");
 let result = JsonSerializer::serialize(&value);

 assert_eq!(&result[..], b"\"hello\"");
 }

 #[test]
 fn test_json_serializer_array() {
 let mut value = JsonValue::array();
 value.push(JsonValue::number(1.0));
 value.push(JsonValue::number(2.0));

 let result = JsonSerializer::serialize(&value);
 assert_eq!(&result[..], b"[1,2]");
 }

 #[test]
 fn test_json_serializer_object() {
 let mut value = JsonValue::object();
 value.insert(b"name", JsonValue::string(b"test"));

 let result = JsonSerializer::serialize(&value);
 assert_eq!(&result[..], b"{\"name\":\"test\"}");
 }
}