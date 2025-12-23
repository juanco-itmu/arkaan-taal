# Arcane

**'n Afrikaanse Programmeertaal** - A programming language with Afrikaans keywords.

Arcane is a functional programming language featuring immutable-by-default variables, first-class functions, pattern matching, algebraic data types, and a stack-based virtual machine.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+) - for VS Code extension

### Build

```bash
# Clone the repository
git clone https://github.com/yourusername/arcane-lang.git
cd arcane-lang

# Build the interpreter and LSP server
cargo build --release
```

### Run a Program

```bash
# Run an Arcane program
cargo run --release -- examples/test_functions.arc

# Or start the interactive REPL
cargo run --release
```

## Language Overview

### Hello World

```arcane
// Dit is 'n kommentaar
druk("Hallo, Wêreld!")
```

### Variables

Arcane uses immutable bindings by default:

```arcane
// Immutable binding (preferred)
laat naam = "Arcane"
laat getal = 42

// Mutable binding (use sparingly)
stel teller = 0
teller = teller + 1
```

### Functions

```arcane
// Function definition
funksie groet(naam) {
    druk("Hallo, " + naam)
}

groet("Wêreld")

// Function with return value
funksie som(a, b) {
    gee a + b
}

laat resultaat = som(3, 4)
druk(resultaat)  // 7

// Recursion
funksie fakulteit(n) {
    as (n <= 1) {
        gee 1
    }
    gee n * fakulteit(n - 1)
}

druk(fakulteit(5))  // 120
```

### Lambda Expressions

```arcane
// Simple lambda
laat dubbel = fn(x) x * 2
druk(dubbel(5))  // 10

// Lambda with block body
laat som = fn(a, b) {
    gee a + b
}

// Closures
laat maak_opteller = fn(n) fn(x) x + n
laat plus_vyf = maak_opteller(5)
druk(plus_vyf(10))  // 15
```

### Control Flow

```arcane
// If-else
laat x = 10

as (x > 5) {
    druk("x is groter as 5")
} anders {
    druk("x is 5 of minder")
}

// While loop
stel teller = 0
terwyl (teller < 5) {
    druk(teller)
    teller = teller + 1
}
```

### Lists

```arcane
laat getalle = [1, 2, 3, 4, 5]

// Access by index
druk(getalle[0])   // 1
druk(getalle[-1])  // 5 (last element)

// List functions
druk(lengte(getalle))      // 5
druk(kop(getalle))         // 1 (first element)
druk(stert(getalle))       // [2, 3, 4, 5]
druk(leeg(getalle))        // vals (false)

// Build lists
laat nuwe = voeg_by(0, getalle)     // prepend: [0, 1, 2, 3, 4, 5]
laat meer = heg_aan(getalle, 6)     // append: [1, 2, 3, 4, 5, 6]
laat saam = ketting([1, 2], [3, 4]) // concat: [1, 2, 3, 4]
laat omgekeer_lys = omgekeer(getalle)  // reverse: [5, 4, 3, 2, 1]
```

### Higher-Order Functions

```arcane
laat getalle = [1, 2, 3, 4, 5]

// Map (kaart)
laat kwadrate = kaart(getalle, fn(x) x * x)
druk(kwadrate)  // [1, 4, 9, 16, 25]

// Filter
laat ewe = filter(getalle, fn(x) x % 2 == 0)
druk(ewe)  // [2, 4]

// Fold/Reduce (vou)
laat som = vou(getalle, 0, fn(acc, x) acc + x)
druk(som)  // 15

// For each (vir_elk)
vir_elk(getalle, fn(x) {
    druk(x)
})
```

### Algebraic Data Types

```arcane
// Define a type with variants
tipe Opsie {
    Niks
    Sommige(waarde)
}

laat opt1 = Sommige(42)
laat opt2 = Niks()

// Pattern matching
laat resultaat = pas(opt1) {
    geval Sommige(x) => x * 2
    geval Niks => 0
}
druk(resultaat)  // 84
```

### Pattern Matching

```arcane
// Binary tree type
tipe Boom {
    Blaar(waarde)
    Tak(links, regs)
}

funksie som(boom) {
    gee pas(boom) {
        geval Blaar(n) => n
        geval Tak(l, r) => som(l) + som(r)
    }
}

laat my_boom = Tak(Blaar(1), Tak(Blaar(2), Blaar(3)))
druk(som(my_boom))  // 6

// Wildcard pattern
laat resultaat = pas(Sommige(100)) {
    geval Sommige(_) => "het iets"
    geval Niks => "het niks"
}
```

## Keyword Reference

| Afrikaans | English     | Purpose                          |
|-----------|-------------|----------------------------------|
| `laat`    | let         | Immutable variable declaration   |
| `stel`    | set         | Mutable variable declaration     |
| `funksie` | function    | Function definition              |
| `fn`      | fn          | Lambda/anonymous function        |
| `gee`     | give/return | Return value from function       |
| `as`      | if          | Conditional                      |
| `anders`  | else        | Else branch                      |
| `terwyl`  | while       | While loop                       |
| `druk`    | print       | Output to console                |
| `waar`    | true        | Boolean true                     |
| `vals`    | false       | Boolean false                    |
| `tipe`    | type        | Define algebraic data type       |
| `pas`     | match       | Pattern matching                 |
| `geval`   | case        | Pattern case                     |

## VS Code Extension

Install the [Arcane Language extension](https://marketplace.visualstudio.com/items?itemName=arcane-lang.arcane-lang) from the VS Code Marketplace for syntax highlighting, code snippets, and LSP features (completions, hover, diagnostics).

## Project Structure

```
arcane-lang/
├── src/
│   ├── main.rs        # CLI entry point & REPL
│   ├── token.rs       # Token definitions
│   ├── lexer.rs       # Tokenizer
│   ├── ast.rs         # Abstract Syntax Tree
│   ├── parser.rs      # Parser
│   ├── compiler.rs    # Bytecode compiler
│   ├── bytecode.rs    # VM instructions
│   ├── vm.rs          # Stack-based VM
│   ├── value.rs       # Runtime values
│   └── lsp/
│       ├── main.rs    # LSP server
│       └── analysis.rs
├── examples/          # Example programs
└── Cargo.toml
```

## Examples

Run the example programs to see Arcane in action:

```bash
cargo run --release -- examples/test_functions.arc   # Functions & recursion
cargo run --release -- examples/test_lambdas.arc     # Lambda expressions
cargo run --release -- examples/test_lists.arc       # List operations
cargo run --release -- examples/test_hof.arc         # Higher-order functions
cargo run --release -- examples/test_pattern.arc     # Pattern matching
cargo run --release -- examples/test_adt.arc         # Algebraic data types
```

## License

MIT
