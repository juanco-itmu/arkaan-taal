"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    // Get the server path from settings or use default
    const config = vscode_1.workspace.getConfiguration('arkaan');
    let serverPath = config.get('serverPath');
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
        vscode_1.window.showErrorMessage('Arkaan LSP server not found. Please build it with `cargo build --release` or set arkaan.serverPath in settings.');
        return;
    }
    const serverExecutable = {
        command: serverPath,
        options: {
            env: process.env,
        },
    };
    const serverOptions = {
        run: serverExecutable,
        debug: serverExecutable,
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'arkaan' }],
        synchronize: {
            fileEvents: vscode_1.workspace.createFileSystemWatcher('**/*.ark'),
        },
    };
    client = new node_1.LanguageClient('arkaanLsp', 'Arkaan Language Server', serverOptions, clientOptions);
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
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map