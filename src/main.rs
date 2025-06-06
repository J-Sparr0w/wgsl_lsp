use serde_json::Value;
use std::collections::HashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        // Log that we're initializing
        self.client
            .log_message(MessageType::INFO, "WGSL LSP server initializing...")
            .await;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "WGSL LSP Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "WGSL LSP server initialized!")
            .await;

        self.client
            .show_message(MessageType::INFO, "WGSL LSP server is ready!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "WGSL LSP server shutting down...")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("File opened: {}", params.text_document.uri),
            )
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // self.client
        //     .log_message(MessageType::INFO, format!("File changed: {:#?}", params))
        //     .await;
        let module = match naga::front::wgsl::parse_str(
            &params
                .content_changes
                .first()
                .expect("content changes was empty")
                .text,
        ) {
            Ok(module) => module,
            Err(parse_err) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        parse_err.emit_to_string(
                            &params
                                .content_changes
                                .first()
                                .expect("content changes was empty")
                                .text,
                        ),
                    )
                    .await;
                return;
            }
        };

        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        );

        match validator.validate(&module) {
            Ok(module_info) => {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("File changed: {:#?}", module_info),
                    )
                    .await;
            }
            Err(err) => {
                self.client
                    .log_message(MessageType::INFO, format!("File changed: {:#?}", err))
                    .await;
            }
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, format!("params: {:#?}", params))
            .await;
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("hello".to_string(), "Hello completion".to_string()),
            CompletionItem::new_simple("world".to_string(), "World completion".to_string()),
        ])))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let position = params.text_document_position_params.position;

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(format!(
                "Hover at line {}, character {}",
                position.line, position.character
            ))),
            range: None,
        }))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
