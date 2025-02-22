# Unit-Testing-Leitfaden

## Überblick

Unit-Tests sind das Fundament unserer Teststrategie. Sie überprüfen die Korrektheit einzelner Funktionen und Komponenten in Isolation. Diese Tests sind schnell, zuverlässig und liefern sofortiges Feedback während der Entwicklung.

## Grundprinzipien

1. **Co-Location mit Quellcode**
   - Tests befinden sich in der gleichen Datei wie der getestete Code
   - Verwendung des `#[cfg(test)]`-Attributs für Testmodule

2. **Unabhängigkeit**
   - Keine externen Abhängigkeiten
   - Keine Datenbankverbindungen
   - Keine Dateisystemoperationen
   - Keine Netzwerkaufrufe

3. **Schnelle Ausführung**
   - Tests sollten innerhalb von Millisekunden abgeschlossen sein
   - Sofortiges Feedback während der Entwicklung
   - Unterstützung für testgetriebene Entwicklung (TDD)

## Teststruktur

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_input_when_processing_then_succeeds() {
        // Arrange
        let input = prepare_test_input();

        // Act
        let result = process_input(input);

        // Assert
        assert_eq!(result, expected_output);
    }
}
```

## Namenskonventionen

- Testnamen sollten dem Muster folgen: `given_[Bedingung]_when_[Aktion]_then_[Ergebnis]`
- Testmodule sollten `tests` heißen
- Hilfsfunktionen sollten beschreibende Namen haben, die ihren Zweck anzeigen

## Testkategorien

### 1. Funktionstests

```rust
#[test]
fn test_add_numbers() {
    assert_eq!(add(2, 2), 4);
}
```

### 2. Fehlerfälle

```rust
#[test]
fn test_division_by_zero() {
    assert!(divide(10, 0).is_err());
}
```

### 3. Randfälle

```rust
#[test]
fn test_empty_input() {
    let result = process_list(vec![]);
    assert_eq!(result.len(), 0);
}
```

## Asynchrone Tests

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_operation().await;
    assert!(result.is_ok());
}
```

## Testdokumentation

```rust
/// Testet die Addition zweier Zahlen
///
/// # Beispiele
///
/// ```
/// let result = add(2, 2);
/// assert_eq!(result, 4);
/// ```
#[test]
fn test_addition() {
    // Testimplementierung
}
```

## Best Practices

1. **Testabdeckung**
   - Streben Sie eine hohe Testabdeckung an
   - Testen Sie sowohl Erfolgs- als auch Fehlerpfade
   - Berücksichtigen Sie Randfälle

2. **Testunabhängigkeit**
   - Jeder Test sollte unabhängig sein
   - Kein gemeinsamer Zustand zwischen Tests
   - Keine Testabhängigkeiten in der Reihenfolge

3. **Testlesbarkeit**
   - Klare Testnamen
   - Gut strukturiertes Arrange-Act-Assert-Muster
   - Aussagekräftige Fehlermeldungen

4. **Testwartung**
   - Regelmäßige Überprüfung und Aktualisierung
   - Entfernen veralteter Tests
   - Tests einfach und fokussiert halten

## Tests ausführen

1. Alle Tests ausführen:

   ```bash
   cargo test
   ```

2. Spezifischen Test ausführen:

   ```bash
   cargo test test_name
   ```

3. Testausgabe anzeigen:

   ```bash
   cargo test -- --show-output
   ```

## Häufige Muster

### Setup-Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TestStruct {
        TestStruct::new()
    }

    #[test]
    fn test_operation() {
        let test_struct = setup();
        // Testimplementierung
    }
}
```

### Test-Utilities

```rust
#[cfg(test)]
mod test_utils {
    pub fn create_test_data() -> Vec<TestData> {
        // Testdaten erstellen
    }
}
```

## Weiterführende Literatur

- [Rust Testing-Dokumentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Testorganisation](../CONTRIBUTING.md#testing)
- [Integrationstests](integration-tests.md)
