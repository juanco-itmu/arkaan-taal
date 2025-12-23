// Test list functionality

// Basic list creation
laat lys = [1, 2, 3, 4, 5]
druk(lys)

// List length
druk(lengte(lys))

// Access by index
druk(lys[0])
druk(lys[2])

// Negative indexing
druk(lys[-1])
druk(lys[-2])

// Empty list
laat leeg_lys = []
druk(leeg_lys)
druk(lengte(leeg_lys))

// Head and tail
laat kop_waarde = kop(lys)
druk(kop_waarde)

laat stert_lys = stert(lys)
druk(stert_lys)

// Check if empty
druk(leeg(lys))
druk(leeg(leeg_lys))

// Prepend (voeg_by)
laat nuwe_lys = voeg_by(0, lys)
druk(nuwe_lys)

// Append (heg_aan)
laat met_einde = heg_aan(lys, 6)
druk(met_einde)

// Concatenate (ketting)
laat lys_a = [1, 2]
laat lys_b = [3, 4]
laat saam = ketting(lys_a, lys_b)
druk(saam)

// Reverse (omgekeer)
laat omgedraai = omgekeer(lys)
druk(omgedraai)

// Strings in lists
laat woorde = ["hallo", "wêreld", "!"]
druk(woorde)
druk(woorde[1])

// Nested lists
laat geneste = [[1, 2], [3, 4], [5, 6]]
druk(geneste)
druk(geneste[1])
druk(geneste[1][0])
