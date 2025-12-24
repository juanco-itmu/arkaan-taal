// Test Algebraic Data Types

// Define a simple Option type
tipe Opsie {
    Niks
    Sommige(waarde)
}

// Create instances
laat geen = Niks()
laat iets = Sommige(42)

druk(geen)
druk(iets)

// Define a Color type with multiple constructors
tipe Kleur {
    Rooi
    Groen
    Blou
    RGB(r, g, b)
}

laat rooi = Rooi()
laat groen = Groen()
laat blou = Blou()
laat perskleur = RGB(128, 0, 128)

druk(rooi)
druk(groen)
druk(blou)
druk(perskleur)

// Define a binary tree type
tipe Boom {
    Blaar(waarde)
    Tak(links, regs)
}

laat boom = Tak(Blaar(1), Tak(Blaar(2), Blaar(3)))
druk(boom)

// Define a linked list type
tipe Lys {
    Leeg
    Kons(kop, stert)
}

laat my_lys = Kons(1, Kons(2, Kons(3, Leeg())))
druk(my_lys)

// Test equality
laat a = Sommige(10)
laat b = Sommige(10)
laat c = Sommige(20)

druk(a == b)
druk(a == c)
druk(Niks() == Niks())

// Store ADTs in lists
laat kleure = [Rooi(), Groen(), Blou()]
druk(kleure)

// ADTs as function arguments and return values
funksie maak_opsie(x) {
    as x > 0 {
        gee Sommige(x)
    } anders {
        gee Niks()
    }
}

druk(maak_opsie(5))
druk(maak_opsie(-1))

// Test HOFs with ADTs
laat opsies = kaart([1, 2, 3], fn(x) Sommige(x * 2))
druk(opsies)
