use dashmap::{DashMap, DashSet};
use naga::front::wgsl::ParseError;
use naga::valid::{ModuleInfo, ValidationError};
use naga::{Module, WithSpan};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

#[derive(Debug, Error)]
enum BackendError {
    #[error("There was a parse error")]
    ParseError(ParseError),
    #[error("There was a validation error")]
    ValidationError(WithSpan<ValidationError>),
}

impl From<ParseError> for BackendError {
    fn from(value: ParseError) -> Self {
        BackendError::ParseError(value)
    }
}
impl From<WithSpan<ValidationError>> for BackendError {
    fn from(value: WithSpan<ValidationError>) -> Self {
        BackendError::ValidationError(value)
    }
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    modules: DashMap<String, Option<ModuleInfo>>,
}

impl Backend {
    const VALIDATION_RESPONSE_TIME: std::time::Duration = std::time::Duration::from_millis(1000);

    pub fn new(client: Client) -> Self {
        Self {
            client,
            modules: DashMap::new(),
        }
    }

    pub async fn on_file_change(&self, params: DidChangeTextDocumentParams) -> anyhow::Result<()> {
        //if it is a didOpen event that triggered the didChange event, then we parse it immediately without any delay.
        let uri = params.text_document.uri.as_str();
        if self.file_was_just_opened(uri) {
            //parse and add the module info to backend
            let source = &params
                .content_changes
                .last()
                .expect("on_file_change was called with no textDocumentContentChangeEvent")
                .text;
            self.parse_wgsl(source).await?;
            return Ok(());
        }
        todo!()
    }

    fn file_was_just_opened(&self, file_uri: &str) -> bool {
        let uri = file_uri;
        // if the entry has a None value, it means the change was a file being opened for the first time
        self.modules.get(uri).is_none()
    }

    async fn on_error(&self, error: BackendError) {}

    async fn parse_wgsl(&self, source: &str) -> std::result::Result<ModuleInfo, BackendError> {
        let module = match naga::front::wgsl::parse_str(source) {
            Ok(module) => module,
            Err(parse_err) => {
                // self.client
                //     .log_message(MessageType::ERROR, parse_err.emit_to_string(source))
                //     .await;
                self.on_error(parse_err.clone().into()).await;
                return Err(BackendError::ParseError(parse_err));
            }
        };

        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        );

        match validator.validate(&module) {
            Ok(module_info) => {
                // self.client
                //     .log_message(
                //         MessageType::INFO,
                //         format!("File changed: {:#?}", module_info),
                //     )
                //     .await;
                return Ok(module_info);
            }
            Err(err) => {
                // self.client
                //     .log_message(MessageType::INFO, format!("File changed: {:#?}", err))
                //     .await;
                self.on_error(err.clone().into()).await;
                return Err(BackendError::ValidationError(err));
            }
        }
    }
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
                inlay_hint_provider: Some(OneOf::Right(
                    InlayHintServerCapabilities::RegistrationOptions(
                        InlayHintRegistrationOptions {
                            inlay_hint_options: InlayHintOptions::default(),
                            text_document_registration_options: TextDocumentRegistrationOptions {
                                document_selector: None,
                            },
                            static_registration_options: Default::default(),
                        },
                    ),
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("Some Diagnostic Identifier".into()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
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
        // self.client
        //     .log_message(
        //         MessageType::INFO,
        //         format!("File opened: {}", params.text_document.uri),
        //     )
        //     .await;
        self.modules
            .insert(params.text_document.uri.as_str().into(), None);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.as_str();
        self.modules.remove(uri);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_file_change(params);
        // self.client
        //     .log_message(MessageType::INFO, format!("File changed: {:#?}", params))
        //     .await;
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

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let diagnostic_report = DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items: vec![Diagnostic {
                        range: Range {
                            start: Position::new(1, 1),
                            end: Position::new(2, 1),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some(String::from("source of the diagnostic")),
                        message: String::from("message of the diagnostic"),
                        related_information: None,
                        tags: None,
                        data: None,
                    }],
                },
            }),
        );
        Ok(diagnostic_report)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri.as_str();

        let mut inlay_hints = Vec::new();
        let inlay = InlayHint {
            position: Position {
                line: 2,
                character: 5,
            },
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip: Some(InlayHintTooltip::String("This is a tooltip".into())),
            padding_left: None,
            padding_right: None,
            data: None,
            label: InlayHintLabel::LabelParts(vec![InlayHintLabelPart {
                value: "Some kind of an inlay hint".into(),
                tooltip: None,
                location: Some(Location {
                    uri: params.text_document.uri.clone(),
                    range: Range {
                        start: Position::new(0, 4),
                        end: Position::new(0, 10),
                    },
                }),
                command: None,
            }]),
        };

        inlay_hints.push(inlay);

        return Ok(Some(inlay_hints));

        // match self.modules.get(uri) {
        //     Some(value) => match *value {
        //         Some(ref module_info) => {
        //         }
        //         None => return Ok(None),
        //     },
        //     None => return Ok(None),
        // }
    }
}
