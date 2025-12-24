// Test Pattern Matching

// Define Option type
tipe Opsie {
    Niks
    Sommige(waarde)
}

// Basic pattern matching with constructor
laat opt1 = Sommige(42)
laat result1 = pas(opt1) {
    geval Sommige(x) => x * 2
    geval Niks => 0
}
druk(result1)

laat opt2 = Niks()
laat result2 = pas(opt2) {
    geval Sommige(x) => x * 2
    geval Niks => 0
}
druk(result2)

// Pattern matching with nested constructors
tipe Boom {
    Blaar(waarde)
    Tak(links, regs)
}

laat boom = Tak(Blaar(1), Blaar(2))

funksie som(b) {
    gee pas(b) {
        geval Blaar(n) => n
        geval Tak(l, r) => som(l) + som(r)
    }
}

druk(som(boom))

// Pattern matching with wildcard
laat result3 = pas(Sommige(100)) {
    geval Sommige(_) => "het iets"
    geval Niks => "het niks"
}
druk(result3)

// Pattern matching in expressions with lambdas
laat dubbel_of_nul = fn(opt) pas(opt) {
    geval Sommige(x) => x * 2
    geval Niks => 0
}

druk(dubbel_of_nul(Sommige(5)))
druk(dubbel_of_nul(Niks()))
