/*
 * Nuva OS
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

// ! DAP servicedeviceImplementation

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use crate::error::SdkError;
use super::protocol::{Message, Request, Response, Event};
use super::DapServer;
use alloc::vec;
use alloc::format;

/// DAP servicedevicerundevice
pub struct DapServerRunner {
 server: DapServer,
}

impl DapServerRunner {
 pub fn new() -> Self {
 Self {
 server: DapServer::new(),
 }
 }

 /// in TCP portuploadrun
 pub fn run_tcp(&mut self, port: u16) -> Result<(), SdkError> {
 let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
 .map_err(|e| SdkError::NetworkError(e.to_string()))?;
 
 let (stream, _) = listener.accept()
 .map_err(|e| SdkError::NetworkError(e.to_string()))?;
 
 self.handle_connection(stream)
 }

 /// processjoin
 fn handle_connection(&mut self, stream: TcpStream) -> Result<(), SdkError> {
 let mut reader = BufReader::new(&stream);
 let mut writer = &stream;
 
 loop {
 // readmessageheader
 let mut header = String::new();
 loop {
 let mut line = String::new();
 reader.read_line(&mut line)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 if line == "\r
" || line.is_empty() {
 break;
 }
 
 header.push_str(&line);
 }
 
 // parseinsidelength
 let content_length = header
 .lines()
 .find(|l| l.starts_with("Content-Length:"))
 .and_then(|l| l.split(':').nth(1))
 .and_then(|s| s.trim().parse::<usize>().ok())
 .unwrap_or(0);
 
 if content_length == 0 {
 break;
 }
 
 // readmessagevolume
 let mut content = vec![0u8; content_length];
 reader.read_exact(&mut content)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 let content = String::from_utf8(content)
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 // parsemessage
 let message: Message = serde_json::from_str(&content)
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 // processrequest
 if let Message::Request(request) = message {
 let response = self.server.handle_request(request)?;
 self.send_response(&mut writer, &response)?;
 }
 }
 
 Ok(())
 }

 /// sendresponse
 fn send_response<W: Write>(&self, writer: &mut W, response: &Response) -> Result<(), SdkError> {
 let content = serde_json::to_string(&Message::Response(response.clone()))
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 let header = format!("Content-Length: {}\r
\r
", content.len());
 
 writer.write_all(header.as_bytes())
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 writer.write_all(content.as_bytes())
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 writer.flush()
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 Ok(())
 }

 /// sendevent
 fn send_event<W: Write>(&self, writer: &mut W, event: &Event) -> Result<(), SdkError> {
 let content = serde_json::to_string(&Message::Event(event.clone()))
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 let header = format!("Content-Length: {}\r
\r
", content.len());
 
 writer.write_all(header.as_bytes())
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 writer.write_all(content.as_bytes())
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 writer.flush()
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 Ok(())
 }
}

impl Default for DapServerRunner {
 fn default() -> Self {
 Self::new()
 }
}