// Simple pattern matching test (no variable bindings)

tipe Opsie {
    Niks
    Sommige(waarde)
}

// Match with wildcards (no binding)
laat opt1 = Sommige(42)
laat result1 = pas(opt1) {
    geval Sommige(_) => "het waarde"
    geval Niks => "leeg"
}
druk(result1)

laat opt2 = Niks()
laat result2 = pas(opt2) {
    geval Sommige(_) => "het waarde"
    geval Niks => "leeg"
}
druk(result2)

// Match with literal patterns
laat n = 1
laat word = pas(n) {
    geval 0 => "nul"
    geval 1 => "een"
    geval 2 => "twee"
    geval _ => "baie"
}
druk(word)

laat n2 = 42
laat word2 = pas(n2) {
    geval 0 => "nul"
    geval 1 => "een"
    geval _ => "ander"
}
druk(word2)
