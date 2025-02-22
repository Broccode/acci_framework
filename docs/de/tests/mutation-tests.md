# Leitfaden für Mutationstests

## Überblick

Mutationstests sind eine Methode zur Bewertung der Qualität Ihrer Testsuite, indem kleine Änderungen (Mutationen) im Code eingeführt werden und überprüft wird, ob Ihre Tests diese Änderungen erkennen können. Dies hilft dabei, Schwachstellen in Ihrer Testabdeckung zu identifizieren und die Testeffektivität zu verbessern.

## Grundkonzepte

1. **Mutationen**
   - Kleine Code-Änderungen
   - Simulierte Fehler
   - Änderungen von Randbedingungen

2. **Mutationsoperatoren**
   - Arithmetische Operatoren
   - Logische Operatoren
   - Kontrollflussänderungen
   - Randbedingungen

3. **Mutationsbewertung**
   - Prozentsatz erkannter Mutationen
   - Effektivität der Testsuite
   - Qualitätsmetrik der Abdeckung

## Verwendung von cargo-mutants

### Grundkonfiguration

```toml
# .mutants.toml
[mutants]
timeout = 300
jobs = 4

paths = [
    "src/core",
    "src/api",
    "src/auth"
]

exclude = [
    "**/tests/*",
    "**/benches/*"
]

operators = [
    "arithmetic",
    "comparison",
    "control_flow",
    "function_calls"
]
```

### Mutationstests ausführen

```bash
# cargo-mutants installieren
cargo install cargo-mutants

# Mutationstests ausführen
cargo mutants

# HTML-Bericht generieren
cargo mutants --reporter html
```

## Mutationskategorien

### 1. Arithmetische Mutationen

```rust
// Ursprünglicher Code
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Mutationen
fn add(a: i32, b: i32) -> i32 {
    a - b    // Mutation 1
    a * b    // Mutation 2
    a / b    // Mutation 3
}
```

### 2. Vergleichsmutationen

```rust
// Ursprünglicher Code
fn ist_gueltiges_alter(alter: u8) -> bool {
    alter >= 18 && alter <= 120
}

// Mutationen
fn ist_gueltiges_alter(alter: u8) -> bool {
    alter > 18 && alter <= 120   // Mutation 1
    alter >= 18 || alter <= 120  // Mutation 2
    alter >= 18 && alter < 120   // Mutation 3
}
```

### 3. Kontrollflussmutationen

```rust
// Ursprünglicher Code
fn verarbeite_liste(items: &[i32]) -> Vec<i32> {
    let mut ergebnis = Vec::new();
    for item in items {
        if *item > 0 {
            ergebnis.push(*item);
        }
    }
    ergebnis
}

// Mutationen
fn verarbeite_liste(items: &[i32]) -> Vec<i32> {
    let mut ergebnis = Vec::new();
    for item in items {
        if *item >= 0 {          // Mutation 1
            ergebnis.push(*item);
        }
        if *item > 0 {           // Mutation 2
            continue;            // Mutation 3
        }
    }
    ergebnis
}
```

## Fortgeschrittene Techniken

### 1. Benutzerdefinierte Mutationsoperatoren

```rust
use cargo_mutants::prelude::*;

#[derive(MutationOperator)]
struct BenutzerdefinierterOperator;

impl Operator for BenutzerdefinierterOperator {
    fn mutate(&self, expr: &Expr) -> Option<Expr> {
        // Implementierung
    }
}
```

### 2. Mutationsfilterung

```rust
// .mutants.toml
[mutants.filter]
paths = ["src/critical"]
min_coverage = 90
exclude_patterns = ["*_generated.rs"]
```

## Best Practices

1. **Testauswahl**
   - Fokus auf kritische Code-Pfade
   - Priorisierung von Bereichen mit hoher Auswirkung
   - Berücksichtigung der Performance-Auswirkungen

2. **Mutationsstrategie**
   - Geeignete Operatoren wählen
   - Angemessene Timeouts setzen
   - Abdeckung und Geschwindigkeit ausbalancieren

3. **Ergebnisanalyse**
   - Überlebende Mutationen überprüfen
   - Testlücken identifizieren
   - Testfälle verbessern

4. **Performance**
   - Parallele Ausführung nutzen
   - Unnötige Mutationen filtern
   - Testlaufzeit optimieren

## Tests ausführen

1. Grundlegende Mutationstests:

   ```bash
   cargo mutants
   ```

2. Mit spezifischer Konfiguration:

   ```bash
   cargo mutants --config custom-mutants.toml
   ```

3. Berichte generieren:

   ```bash
   cargo mutants --reporter json --output mutations.json
   ```

## Häufige Muster

### Testverbesserung

```rust
// Vor Mutationstests
#[test]
fn test_verarbeite_positiv() {
    let ergebnis = verarbeite_zahlen(&[1, 2, 3]);
    assert_eq!(ergebnis.len(), 3);
}

// Nach Mutationstests
#[test]
fn test_verarbeite_positiv() {
    let ergebnis = verarbeite_zahlen(&[1, 2, 3]);
    assert_eq!(ergebnis.len(), 3);
    assert_eq!(ergebnis, vec![1, 2, 3]);  // Stärkere Überprüfung
}
```

### Mutationsresistenz

```rust
// Mutationsanfälliger Code
fn validiere_bereich(wert: i32) -> bool {
    wert >= 0 && wert <= 100
}

// Mutationsresistenter Code
fn validiere_bereich(wert: i32) -> bool {
    let min = 0;
    let max = 100;
    wert >= min && wert <= max
}
```

## Integration mit CI/CD

### 1. GitHub Actions

```yaml
name: Mutationstests

on: [push, pull_request]

jobs:
  mutants:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: cargo-mutants installieren
        run: cargo install cargo-mutants
      - name: Mutationstests ausführen
        run: cargo mutants --reporter github
```

### 2. Qualitäts-Gates

```rust
// mutation-check.rs
fn main() {
    let bericht = parse_mutation_report("mutations.json");
    if bericht.score < 0.80 {
        std::process::exit(1);
    }
}
```

## Weiterführende Literatur

- [Mutationstest-Buch](https://mutation-testing.org/)
- [cargo-mutants-Dokumentation](https://docs.rs/cargo-mutants)
- [Testqualitätsmetriken](../testing/METRICS.md)
- [CI-Integrationsleitfaden](../ci/MUTATION_TESTING.md)
