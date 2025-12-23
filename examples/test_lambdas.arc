// Test lambda expressions

// Simple lambda with expression body
stel dubbel = fn(x) x * 2
druk(dubbel(5))

// Lambda with block body
stel som = fn(a, b) {
    gee a + b
}
druk(som(3, 4))

// Lambda as argument (higher-order function test)
funksie pas_toe(f, x) {
    gee f(x)
}

druk(pas_toe(fn(n) n * n, 4))

// Nested lambda
stel maak_opteller = fn(n) fn(x) x + n
stel plus_vyf = maak_opteller(5)
druk(plus_vyf(10))
