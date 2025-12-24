import * as path from 'path';
import * as fs from 'fs';
import { workspace, ExtensionContext, window } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // Get the server path from settings or use default
    const config = workspace.getConfiguration('arkaan');
    let serverPath = config.get<string>('serverPath');

    if (!serverPath || serverPath === '') {
        // Try to find the server in common locations
        const possiblePaths = [
            // Development: built with cargo
            path.join(context.extensionPath, '..', 'target', 'release', 'arkaan-lsp'),
            path.join(context.extensionPath, '..', 'target', 'debug', 'arkaan-lsp'),
            // Installed globally
            'arkaan-lsp',
            // In the extension folder
            path.join(context.extensionPath, 'bin', 'arkaan-lsp'),
        ];

        for (const p of possiblePaths) {
            if (p === 'arkaan-lsp' || fs.existsSync(p)) {
                serverPath = p;
                break;
            }
        }
    }

    if (!serverPath) {
        window.showErrorMessage(
            'Arkaan LSP server not found. Please build it with `cargo build --release` or set arkaan.serverPath in settings.'
        );
        return;
    }

    const serverExecutable: Executable = {
        command: serverPath,
        options: {
            env: process.env,
        },
    };

    const serverOptions: ServerOptions = {
        run: serverExecutable,
        debug: serverExecutable,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'arkaan' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.ark'),
        },
    };

    client = new LanguageClient(
        'arkaanLsp',
        'Arkaan Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client (and server)
    client.start();

    context.subscriptions.push({
        dispose: () => {
            if (client) {
                client.stop();
            }
        },
    });
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
