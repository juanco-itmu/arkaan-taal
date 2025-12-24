// Test global variables
stel x = 10
druk(x)

// Test local variables in a block
{
    stel y = 20
    druk(y)
    druk(x + y)
}

// x is still accessible
druk(x)

// Nested scopes
{
    stel a = 1
    {
        stel b = 2
        druk(a + b)
    }
    // b is out of scope here
    druk(a)
}

// Variable shadowing
stel naam = "buite"
{
    stel naam = "binne"
    druk(naam)
}
druk(naam)
