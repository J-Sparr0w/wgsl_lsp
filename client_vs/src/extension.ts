import * as path from 'path';
import * as vscode from 'vscode';
import * as fs from 'fs';

import {
  Disposable,
  Executable,
  InitializeError,
  LanguageClient,
  LanguageClientOptions,
  ResponseError,
  RevealOutputChannelOn,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;
const DEBUG_SERVER_PATH = "../../target/debug/wgsl_lsp.exe";
const id = 'wgsl';
const name = 'WGSL Language Server trace';
const documentSelector = [
  { scheme: 'file', language: 'wgsl' },
];
const outputChannel = vscode.window.createOutputChannel(name);
let initializationError: ResponseError<InitializeError> | Error | undefined = undefined;
let crashCount = 0;


function getServerPath(context: vscode.ExtensionContext): null | string {
  if (process.env.SERVER_PATH) {
    return process.env.SERVER_PATH;
  }
  // let serverDir = context.asAbsolutePath(path.join('..', 'target', 'release', 'wgsl_lsp.exe'));
  let serverDir = context.asAbsolutePath(path.join('..', 'target', 'debug', 'wgsl_lsp.exe'));
  // console.log('serverDir: ', serverDir);
  if (!fs.existsSync(serverDir)) { return null; }
  return serverDir;
}

function registerCommands(context: vscode.ExtensionContext) {
  // Command to restart the language server
  const restartCmd = vscode.commands.registerCommand('wgsl-lsp.restart', () => {
    if (client) {
      client.restart();
      vscode.window.showInformationMessage('LSP Client restarted');
    }
  });

  // Command to show server status
  const statusCmd = vscode.commands.registerCommand('wgsl-lsp.status', () => {
    if (client) {
      const state = client.state;
      vscode.window.showInformationMessage(`LSP Client State: ${state}`);
    }
  });

  context.subscriptions.push(restartCmd, statusCmd);
}


export async function activate(context: vscode.ExtensionContext) {
  if (process.env.WGSL_TEST === 'true') {
    // setupMockServer();
  }
  // let serverOptions: ServerOptions = {
  // 	command: serverDir, transport: TransportKind.stdio, args: []
  // };
  const serverExecutable = getServerPath(context);
  // Check if server exists
  if (!serverExecutable) {
    vscode.window.showErrorMessage('WGSL LSP server not found. Please build and configure the server path.');
    return;
  }

  const run: Executable = {
    command: serverExecutable,
    transport: TransportKind.stdio,
    options: {
      env: {
        ...process.env,
        RUST_LOG: "debug",
      },
    },
  };
  const debug_run: Executable = {
    command: serverExecutable,
    transport: TransportKind.stdio,
    options: {
      env: {
        ...process.env,
        RUST_LOG: "debug",
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: debug_run,
  };

  let clientOptions: LanguageClientOptions = {
    documentSelector: documentSelector,
    progressOnInitialization: true,
    // synchronize: {
    //   fileEvents: vscode.workspace.createFileSystemWatcher("**/*.wgsl"),
    // },
    outputChannel,
    // revealOutputChannelOn: RevealOutputChannelOn.Never // for production
  };

  // Create the language client and start the client.
  client = new LanguageClient("wgsl-language-server", "WGSL Language Server", serverOptions, clientOptions);
  registerCommands(context);

  // activateInlayHints(context);

  console.log('Starting LSP client...');
  client.start().then(() => {
    console.log('LSP client started successfully');
  }).catch((error) => {
    console.error('Failed to start LSP client:', error);
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}



// export function activateInlayHints(ctx: vscode.ExtensionContext) {
//   const maybeUpdater = {
//     hintsProvider: null as Disposable | null,
//     updateHintsEventEmitter: new vscode.EventEmitter<void>(),

//     async onConfigChange() {
//       this.dispose();

//       const event = this.updateHintsEventEmitter.event;
//       this.hintsProvider = vscode.languages.registerInlayHintsProvider(
//         { scheme: "file", language: "wgsl" },
//         new (class implements vscode.InlayHintsProvider {
//           onDidChangeInlayHints = event;
//           resolveInlayHint(hint: vscode.InlayHint, token: vscode.CancellationToken): vscode.ProviderResult<vscode.InlayHint> {
//             const ret = {
//               // label: hint.label,
//               ...hint,
//             };
//             return ret;
//           }
//           async provideInlayHints(
//             document: vscode.TextDocument,
//             range: vscode.Range,
//             token: vscode.CancellationToken
//           ): Promise<vscode.InlayHint[]> {
//             const hints = (await client
//               .sendRequest("custom/inlay_hint", { path: document.uri.toString() })
//               .catch(err => null)) as [number, number, string][];
//             if (hints === null) {
//               return [];
//             } else {
//               return hints.map(item => {
//                 const [start, end, label] = item;
//                 let startPosition = document.positionAt(start);
//                 let endPosition = document.positionAt(end);
//                 return {
//                   position: endPosition,
//                   paddingLeft: true,
//                   label: [
//                     {
//                       value: `${label}`,
//                       // location: {
//                       //   uri: document.uri,
//                       //   range: new Range(1, 0, 1, 0)
//                       // }
//                       command: {
//                         title: "hello world",
//                         command: "helloworld.helloWorld",
//                         arguments: [document.uri],
//                       },
//                     },
//                   ],
//                 };
//               });
//             }
//           }
//         })()
//       );
//     },

//     onDidChangeTextDocument({ contentChanges, document }: vscode.TextDocumentChangeEvent) {
//       // debugger
//       // this.updateHintsEventEmitter.fire();
//     },

//     dispose() {
//       this.hintsProvider?.dispose();
//       this.hintsProvider = null;
//       this.updateHintsEventEmitter.dispose();
//     },
//   };

//   vscode.workspace.onDidChangeConfiguration(maybeUpdater.onConfigChange, maybeUpdater, ctx.subscriptions);
//   vscode.workspace.onDidChangeTextDocument(maybeUpdater.onDidChangeTextDocument, maybeUpdater, ctx.subscriptions);

//   maybeUpdater.onConfigChange().catch(console.error);
// }
