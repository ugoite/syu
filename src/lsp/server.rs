// FEAT-LSP-001
// REQ-CORE-001

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};

use super::handlers::LspHandlers;
use super::protocol::{Message, Notification, Request, Response, ResponseError};

pub(crate) struct LspServer {
    handlers: LspHandlers,
    should_exit: bool,
}

impl LspServer {
    pub(crate) fn new() -> Self {
        Self {
            handlers: LspHandlers::new(),
            should_exit: false,
        }
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdin = BufReader::new(stdin.lock());
        let mut stdout = std::io::stdout();

        loop {
            if self.should_exit {
                break;
            }

            match self.read_message(&mut stdin)? {
                Some(msg) => {
                    if let Some(response) = self.handle_message(msg)? {
                        self.write_message(&mut stdout, &response)?;
                    }
                }
                None => break,
            }
        }

        Ok(())
    }

    fn read_message<R: BufRead>(&self, reader: &mut R) -> Result<Option<Message>> {
        let mut headers = Vec::new();
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Ok(None);
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            headers.push(trimmed.to_string());
        }

        let content_length = headers
            .iter()
            .find_map(|h| {
                h.strip_prefix("Content-Length: ")
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .context("missing Content-Length header")?;

        let mut content = vec![0u8; content_length];
        reader.read_exact(&mut content)?;

        let message: Message = serde_json::from_slice(&content)?;
        Ok(Some(message))
    }

    fn write_message<W: Write>(&self, writer: &mut W, message: &Message) -> Result<()> {
        let json = serde_json::to_string(message)?;
        write!(writer, "Content-Length: {}\r\n\r\n{}", json.len(), json)?;
        writer.flush()?;
        Ok(())
    }

    fn handle_message(&mut self, message: Message) -> Result<Option<Message>> {
        match message {
            Message::Request(req) => Ok(Some(Message::Response(self.handle_request(req)?))),
            Message::Notification(notif) => {
                self.handle_notification(notif)?;
                Ok(None)
            }
            Message::Response(_) => Ok(None),
        }
    }

    fn handle_request(&mut self, request: Request) -> Result<Response> {
        let result = match request.method.as_str() {
            "initialize" => {
                let params = request
                    .params
                    .map(serde_json::from_value)
                    .transpose()?
                    .unwrap_or_default();
                self.handlers.handle_initialize(params)
            }
            "shutdown" => self.handlers.handle_shutdown(),
            "textDocument/hover" => {
                let params = request
                    .params
                    .ok_or_else(|| anyhow::anyhow!("missing params"))?;
                let params = serde_json::from_value(params)?;
                let hover = self.handlers.handle_hover(params)?;
                Ok(serde_json::to_value(hover)?)
            }
            _ => Err(anyhow::anyhow!("method not found: {}", request.method)),
        };

        match result {
            Ok(value) => Ok(Response {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            }),
            Err(e) => Ok(Response {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(ResponseError {
                    code: super::protocol::error_codes::INTERNAL_ERROR,
                    message: e.to_string(),
                    data: None,
                }),
            }),
        }
    }

    fn handle_notification(&mut self, notification: Notification) -> Result<()> {
        match notification.method.as_str() {
            "initialized" => self.handlers.handle_initialized()?,
            "exit" => {
                self.should_exit = true;
            }
            _ => {}
        }
        Ok(())
    }
}
