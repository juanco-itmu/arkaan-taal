// Test immutable bindings with laat

// laat creates immutable binding
laat x = 10
druk(x)

// This should fail: trying to reassign immutable variable
// x = 20
// druk(x)

// stel creates mutable binding
stel y = 10
druk(y)
y = 20
druk(y)

// Functions in scope are also immutable
funksie groet(naam) {
    druk("Hallo, " + naam)
}
groet("Wereld")

// Function parameters are immutable
funksie probeer(x) {
    // x = 42  // Would fail: cannot reassign immutable parameter
    druk(x)
}
probeer(100)

druk("Toets geslaag!")
