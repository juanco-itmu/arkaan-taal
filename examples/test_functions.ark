// Test basic function
funksie groet(naam) {
    druk("Hallo, " + naam)
}

groet("Wereld")

// Test function with return value
funksie som(a, b) {
    gee a + b
}

stel resultaat = som(3, 4)
druk("3 + 4 = " + resultaat)

// Test recursion (factorial)
funksie fakulteit(n) {
    as n <= 1 {
        gee 1
    }
    gee n * fakulteit(n - 1)
}

druk("5! = " + fakulteit(5))

// Test nested calls
funksie dubbel(x) {
    gee x * 2
}

funksie verdubbel_som(a, b) {
    gee dubbel(som(a, b))
}

druk("dubbel(3+4) = " + verdubbel_som(3, 4))
