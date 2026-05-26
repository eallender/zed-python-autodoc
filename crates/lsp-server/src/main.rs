mod docgen;

use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

/// Simple in-memory store for open document contents, keyed by URI.
#[derive(Default, Debug)]
struct DocumentStore {
    docs: Mutex<HashMap<String, String>>,
}

impl DocumentStore {
    fn update(&self, uri: &str, text: String) {
        self.docs
            .lock()
            .expect("DocumentStore lock poisoned")
            .insert(uri.to_string(), text);
    }
    fn close(&self, uri: &str) {
        self.docs
            .lock()
            .expect("DocumentStore lock poisoned")
            .remove(uri);
    }
    fn get(&self, uri: &str) -> Option<String> {
        self.docs
            .lock()
            .expect("DocumentStore lock poisoned")
            .get(uri)
            .cloned()
    }
}

#[derive(Debug)]
struct Backend {
    client: Client,
    store: DocumentStore,
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Trigger completions when the user types `"`
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!['"'.to_string()]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                // FULL sync so we always have the latest document text
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "python-autodoc LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    // -- Document sync --

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.store.update(
            &params.text_document.uri.to_string(),
            params.text_document.text,
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if params.content_changes.len() > 1 {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "python-autodoc: received incremental sync; expected FULL",
                )
                .await;
        }
        // FULL sync — last change event contains the whole document
        if let Some(change) = params.content_changes.into_iter().last() {
            self.store
                .update(&params.text_document.uri.to_string(), change.text);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.store.close(&params.text_document.uri.to_string());
    }

    // -- Completions --

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let position = params.text_document_position.position;
        let uri = params.text_document_position.text_document.uri.to_string();

        let Some(text) = self.store.get(&uri) else {
            return Ok(None);
        };

        let lines: Vec<&str> = text.lines().collect();
        let cursor_line = position.line as usize;
        let cursor_char = position.character as usize;

        // Only fire when the text typed so far on this line is exactly `"""`
        let Some(current_line) = lines.get(cursor_line) else {
            return Ok(None);
        };
        let before_cursor = &current_line[..cursor_char.min(current_line.len())];
        if before_cursor.trim() != r#"""""# {
            return Ok(None);
        }

        // Find the nearest def/class above the cursor line
        let Some(def_source) = docgen::find_definition_above(&lines, cursor_line) else {
            return Ok(None);
        };

        // Build the PEP 257 docstring body
        // Pass all lines so we can look for raise statements in the function body
        let Some(body) = docgen::generate_docstring(&def_source, &lines, cursor_line) else {
            return Ok(None);
        };

        // Preserve the indentation of the opening `"""`
        let indent_len = current_line.len() - current_line.trim_start().len();
        let indent = " ".repeat(indent_len);

        // A body without a leading newline is a PEP 257 one-liner (e.g. class summaries).
        let snippet = if !body.starts_with('\n') {
            format!("{}\"\"\"", body)
        } else {
            // Prefix each non-empty body line with the current indentation so the
            // snippet lands at the right column regardless of editor auto-indent.
            let indented_body = body
                .lines()
                .map(|line| {
                    if line.is_empty() {
                        String::new()
                    } else {
                        format!("{}{}", indent, line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}\n{}\"\"\"", indented_body, indent)
        };

        // If the editor auto-paired a closing `"""` after the cursor, replace
        // it so we don't end up with `""""""`.
        let after_cursor = &current_line[cursor_char..];
        let end_char = if after_cursor.starts_with("\"\"\"") {
            cursor_char + 3
        } else {
            cursor_char
        };
        let end_position = Position {
            line: position.line,
            character: end_char as u32,
        };

        let item = CompletionItem {
            label: "\"\"\"  Generate PEP 257 docstring".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range: Range {
                    start: position,
                    end: end_position,
                },
                new_text: snippet,
            })),
            detail: Some("python-autodoc".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Generate a PEP 257 docstring from the function or class above.".to_string(),
            })),
            preselect: Some(true),
            sort_text: Some("0000".to_string()),
            ..Default::default()
        };

        Ok(Some(CompletionResponse::Array(vec![item])))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        store: DocumentStore::default(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
