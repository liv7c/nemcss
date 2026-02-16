import * as path from "path";
import * as fs from "fs";
import { workspace, ExtensionContext, window } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

function getLspBinaryPath(context: ExtensionContext): string | null {
  const config = workspace.getConfiguration("nemcss");
  const customPath = config.get<string>("lspPath");

  if (customPath) {
    // Normalize and validate the path to prevent path traversal attacks
    const normalizedPath = path.normalize(customPath);

    // Prevent path traversal (e.g., "../../../etc/passwd")
    if (normalizedPath.includes("..")) {
      window.showWarningMessage(
        `NemCSS: Invalid LSP path (contains ..): ${customPath}`,
      );
      return null;
    }

    if (fs.existsSync(normalizedPath)) {
      return normalizedPath;
    }

    window.showWarningMessage(
      `NemCSS: Custom LSP path not found: ${customPath}`,
    );
  }

  const platform = process.platform;
  const arch = process.arch;
  let binaryName = "lsp";

  if (platform === "win32") {
    binaryName = `lsp-${platform}-${arch}.exe`;
  } else {
    binaryName = `lsp-${platform}-${arch}`;
  }

  const binaryPath = path.join(context.extensionPath, "bin", binaryName);

  if (fs.existsSync(binaryPath)) {
    return binaryPath;
  }

  window.showErrorMessage(
    `NemCSS: LSP binary not found at ${binaryPath}. Please check your installation`,
  );
  return null;
}

export function activate(context: ExtensionContext) {
  const lspPath = getLspBinaryPath(context);

  // cannot start without the binary
  if (!lspPath) {
    return;
  }

  const serverOptions: ServerOptions = {
    command: lspPath,
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "css" },
      { scheme: "file", language: "scss" },
      { scheme: "file", language: "sass" },
      { scheme: "file", language: "less" },

      { scheme: "file", language: "html" },
      { scheme: "file", language: "php" },

      { scheme: "file", language: "javascript" },
      { scheme: "file", language: "javascriptreact" },
      { scheme: "file", language: "typescript" },
      { scheme: "file", language: "typescriptreact" },

      { scheme: "file", language: "vue" },
      { scheme: "file", language: "svelte" },
      { scheme: "file", language: "astro" },
    ],
  };

  client = new LanguageClient(
    "nemcss",
    "NemCSS LSP",
    serverOptions,
    clientOptions,
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }

  return client.stop();
}
