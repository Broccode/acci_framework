# Leitfaden für eigenschaftsbasiertes Testen

## Überblick

Eigenschaftsbasiertes Testen ist eine Testmethodik, bei der statt einzelner Testfälle Eigenschaften definiert werden, die der Code erfüllen muss, und das Test-Framework automatisch Testfälle generiert. Dieser Ansatz kann Randfälle finden, an die man beim Schreiben traditioneller Unit-Tests möglicherweise nicht denkt.

## Grundkonzepte

1. **Eigenschaften**
   - Invarianten, die für alle Eingaben gelten müssen
   - Beziehungen zwischen Ein- und Ausgaben
   - Zustandsübergänge und deren Auswirkungen

2. **Generatoren**
   - Automatische Testdatengenerierung
   - Generierung benutzerdefinierter Datentypen
   - Schrumpfen auf minimale Fehlerfälle

3. **Testfälle**
   - Automatisch generierte Eingaben
   - Mehrere Testdurchläufe mit verschiedenen Werten
   - Reproduzierbare Fehler

## Verwendung von Proptest

### Grundlegendes Beispiel

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_addition_kommutativ(a in 0..100i32, b in 0..100i32) {
        assert_eq!(a + b, b + a);
    }
}
```

### Benutzerdefinierte Typen

```rust
#[derive(Debug)]
struct Benutzer {
    name: String,
    alter: u8,
}

prop_compose! {
    fn zufaelliger_benutzer()(
        name in "[A-Za-z]{1,10}",
        alter in 0..120u8
    ) -> Benutzer {
        Benutzer { name, alter }
    }
}

proptest! {
    #[test]
    fn test_benutzer_erstellung(benutzer in zufaelliger_benutzer()) {
        assert!(benutzer.alter <= 120);
        assert!(!benutzer.name.is_empty());
    }
}
```

## Testkategorien

### 1. Numerische Eigenschaften

```rust
proptest! {
    #[test]
    fn test_absolutwert(x in -1000i32..1000i32) {
        let abs_x = x.abs();
        assert!(abs_x >= 0);
        assert_eq!(abs_x.abs(), abs_x);
    }
}
```

### 2. String-Eigenschaften

```rust
proptest! {
    #[test]
    fn test_string_umkehrung(s in ".*") {
        let umgekehrt = s.chars().rev().collect::<String>();
        let doppelt_umgekehrt = umgekehrt.chars().rev().collect::<String>();
        assert_eq!(s, doppelt_umgekehrt);
    }
}
```

### 3. Sammlungs-Eigenschaften

```rust
proptest! {
    #[test]
    fn test_vec_sortierung(mut vec in prop::collection::vec(0..100i32, 0..100)) {
        vec.sort();
        for i in 1..vec.len() {
            assert!(vec[i-1] <= vec[i]);
        }
    }
}
```

## Fortgeschrittene Techniken

### 1. Zustandsmaschinen-Tests

```rust
#[derive(Debug)]
enum Aktion {
    Push(i32),
    Pop,
}

proptest! {
    #[test]
    fn test_stack_operationen(aktionen in prop::collection::vec(
        prop_oneof![
            Just(Aktion::Pop),
            (0..100i32).prop_map(Aktion::Push)
        ],
        0..100
    )) {
        let mut stack = Vec::new();
        
        for aktion in aktionen {
            match aktion {
                Aktion::Push(x) => stack.push(x),
                Aktion::Pop => { stack.pop(); }
            }
            
            // Invariante: Stackgröße ist nie negativ
            assert!(stack.len() >= 0);
        }
    }
}
```

### 2. Asynchrone Eigenschaftstests

```rust
proptest! {
    #[test]
    fn test_async_operation(eingabe in ".*") {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let ergebnis = async_operation(&eingabe).await;
            assert!(ergebnis.is_ok());
        });
    }
}
```

## Best Practices

1. **Eigenschaftsauswahl**
   - Wählen Sie Eigenschaften, die immer wahr sind
   - Fokussieren Sie sich auf Invarianten und Beziehungen
   - Berücksichtigen Sie Randfälle und Grenzen

2. **Generator-Design**
   - Erstellen Sie fokussierte Generatoren
   - Verwenden Sie angemessene Wertebereiche
   - Berücksichtigen Sie Domäneneinschränkungen

3. **Testkonfiguration**
   - Setzen Sie angemessene Testfallanzahlen
   - Konfigurieren Sie Timeout-Werte
   - Behandeln Sie nicht-deterministisches Verhalten

4. **Fehleranalyse**
   - Untersuchen Sie geschrumpfte Testfälle
   - Suchen Sie nach Mustern in Fehlern
   - Dokumentieren Sie entdeckte Eigenschaften

## Tests ausführen

1. Alle Eigenschaftstests ausführen:

   ```bash
   cargo test
   ```

2. Mit mehr Testfällen ausführen:

   ```bash
   PROPTEST_CASES=10000 cargo test
   ```

3. Mit Debug-Ausgabe ausführen:

   ```bash
   RUST_LOG=debug cargo test
   ```

## Häufige Muster

### Benutzerdefinierte Generatoren

```rust
prop_compose! {
    fn gueltige_email()(
        local in "[a-zA-Z0-9._%+-]{1,64}",
        domain in "[a-zA-Z0-9.-]{1,255}"
    ) -> String {
        format!("{}@{}", local, domain)
    }
}
```

### Eigenschaftskomposition

```rust
fn ist_sortiert<T: Ord>(slice: &[T]) -> bool {
    slice.windows(2).all(|w| w[0] <= w[1])
}

proptest! {
    #[test]
    fn test_sort_idempotent(mut vec in prop::collection::vec(0..100i32, 0..100)) {
        vec.sort();
        assert!(ist_sortiert(&vec));
        
        let sortiert = vec.clone();
        vec.sort();
        assert_eq!(vec, sortiert);
    }
}
```

## Weiterführende Literatur

- [Proptest-Dokumentation](https://docs.rs/proptest)
- [QuickCheck für Rust](https://docs.rs/quickcheck)
- [Eigenschaftsbasiertes Testen in der Praxis](https://blog.rust-lang.org/2020/01/07/proptest.html)
- [Teststrategien-Leitfaden](../TESTING.md)
