// Test Tail Call Optimization

// Tail-recursive factorial
funksie faktoriaal(n, acc) {
    as n <= 1 {
        gee acc
    } anders {
        gee faktoriaal(n - 1, n * acc)
    }
}

druk(faktoriaal(5, 1))   // 120
druk(faktoriaal(10, 1))  // 3628800

// Tail-recursive sum
funksie som(n, acc) {
    as n <= 0 {
        gee acc
    } anders {
        gee som(n - 1, acc + n)
    }
}

druk(som(100, 0))  // 5050

// Test deep recursion - would stack overflow without TCO
funksie tel_af(n) {
    as n <= 0 {
        gee "klaar!"
    } anders {
        gee tel_af(n - 1)
    }
}

druk(tel_af(10000))  // Should work with TCO

// Tail call with lambda
laat vermenigvuldig = fn(n, acc) {
    as n <= 1 {
        gee acc
    } anders {
        gee vermenigvuldig(n - 1, n * acc)
    }
}

druk(vermenigvuldig(6, 1))  // 720

// Mutual recursion (indirect tail calls)
funksie is_ewe(n) {
    as n == 0 {
        gee waar
    } anders {
        gee is_onewe(n - 1)
    }
}

funksie is_onewe(n) {
    as n == 0 {
        gee vals
    } anders {
        gee is_ewe(n - 1)
    }
}

druk(is_ewe(100))   // waar
druk(is_onewe(100)) // vals
