# Arcane Language Support for VS Code

Syntax highlighting and code snippets for Arcane - 'n Afrikaanse programmeertaal.

## Features

- Syntax highlighting for `.arc` files
- Code snippets for common patterns
- Bracket matching and auto-closing
- Comment toggling with `Ctrl+/`

## Snippets

| Prefix | Description |
|--------|-------------|
| `stel` | Variable declaration |
| `druk` | Print statement |
| `as` | If statement |
| `asanders` | If-else statement |
| `terwyl` | While loop |
| `waar` | Boolean true |
| `vals` | Boolean false |
| `vir` | Counted loop pattern |
| `som` | Sum calculation pattern |

## Installation

### Option 1: Symlink (Development)
```bash
# Linux/macOS
ln -s /path/to/arcane-lang/vscode-arcane ~/.vscode/extensions/arcane-lang

# Windows (PowerShell as Admin)
New-Item -ItemType SymbolicLink -Path "$env:USERPROFILE\.vscode\extensions\arcane-lang" -Target "C:\path\to\arcane-lang\vscode-arcane"
```

### Option 2: Copy
Copy the `vscode-arcane` folder to your VS Code extensions directory:
- Linux: `~/.vscode/extensions/`
- macOS: `~/.vscode/extensions/`
- Windows: `%USERPROFILE%\.vscode\extensions\`

Then restart VS Code.

## Example

```arcane
// Bereken die som van 1 tot 10
stel x = 10
stel som = 0

terwyl (x > 0) {
    stel som = som + x
    stel x = x - 1
}

druk(som)
```
