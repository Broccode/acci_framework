# Leitfaden für Performance-Tests

## Überblick

Performance-Tests sind entscheidend, um sicherzustellen, dass unsere Anwendung die Leistungsanforderungen erfüllt. Dieser Leitfaden behandelt verschiedene Arten von Performance-Tests, Werkzeuge und Best Practices für die Messung und Optimierung der Anwendungsleistung.

## Grundkonzepte

1. **Benchmark-Tests**
   - Ausführungszeit messen
   - Verschiedene Implementierungen vergleichen
   - Leistungsregressionen verfolgen

2. **Lasttests**
   - Mehrere Benutzer simulieren
   - System unter Last testen
   - Antwortzeiten messen

3. **Stresstests**
   - Systemgrenzen testen
   - Ressourcenerschöpfungsszenarien
   - Wiederherstellungsverhalten

## Verwendung von Criterion.rs

### Grundlegender Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn bench_fibonacci(c: &mut Criterion) {
    c.bench_function("fibonacci 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, bench_fibonacci);
criterion_main!(benches);
```

### Parametrisierte Benchmarks

```rust
fn bench_sortierung(c: &mut Criterion) {
    let mut group = c.benchmark_group("sortierung");
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("vec_sort", size),
            size,
            |b, &size| {
                let mut vec: Vec<i32> = (0..size).collect();
                b.iter(|| {
                    vec.shuffle(&mut rand::thread_rng());
                    vec.sort();
                })
            },
        );
    }
    group.finish();
}
```

## Testkategorien

### 1. CPU-Leistung

```rust
fn bench_berechnung(c: &mut Criterion) {
    c.bench_function("komplexe_berechnung", |b| {
        b.iter(|| {
            // CPU-intensive Operation
            fuehre_komplexe_berechnung_aus()
        })
    });
}
```

### 2. Speichernutzung

```rust
fn bench_speicher(c: &mut Criterion) {
    c.bench_function("speicher_allokation", |b| {
        b.iter(|| {
            let mut daten = Vec::with_capacity(1000);
            for i in 0..1000 {
                daten.push(i);
            }
            black_box(daten)
        })
    });
}
```

### 3. I/O-Leistung

```rust
fn bench_io(c: &mut Criterion) {
    c.bench_function("datei_lesen", |b| {
        b.iter(|| {
            let mut datei = File::open("test_daten.txt").unwrap();
            let mut puffer = String::new();
            datei.read_to_string(&mut puffer).unwrap();
            black_box(puffer)
        })
    });
}
```

## Fortgeschrittene Techniken

### 1. Asynchrone Benchmarks

```rust
fn bench_async(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("async_operation", |b| {
        b.to_async(&rt).iter(|| async {
            let ergebnis = async_operation().await;
            black_box(ergebnis)
        })
    });
}
```

### 2. Benutzerdefinierte Messungen

```rust
fn bench_custom(c: &mut Criterion) {
    let mut group = c.benchmark_group("benutzerdefinierte_metriken");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    group.bench_function("operation", |b| {
        b.iter(|| operation_zum_messen())
    });
    group.finish();
}
```

## Best Practices

1. **Benchmark-Umgebung**
   - Dedizierte Hardware verwenden
   - Hintergrundprozesse minimieren
   - Konsistenter Systemzustand

2. **Testdesign**
   - Realistische Arbeitslasten
   - Repräsentative Datensätze
   - Angemessene Aufwärmphasen

3. **Metrikerfassung**
   - Durchsatzmessungen
   - Latenzverteilungen
   - Ressourcenauslastung

4. **Analyse**
   - Statistische Signifikanz
   - Erkennung von Leistungsregressionen
   - Trendanalyse

## Tests ausführen

1. Alle Benchmarks ausführen:

   ```bash
   cargo bench
   ```

2. Spezifischen Benchmark ausführen:

   ```bash
   cargo bench --bench mein_benchmark
   ```

3. Mit benutzerdefinierter Konfiguration ausführen:

   ```bash
   CRITERION_DEBUG=1 cargo bench
   ```

## Häufige Muster

### Setup und Teardown

```rust
fn bench_mit_setup(c: &mut Criterion) {
    c.bench_function("operation_mit_setup", |b| {
        b.iter_with_setup(
            || setup_testdaten(),
            |testdaten| {
                fuehre_operation_aus(testdaten)
            }
        )
    });
}
```

### Implementierungen vergleichen

```rust
fn bench_vergleich(c: &mut Criterion) {
    let mut group = c.benchmark_group("implementierungen");
    group.bench_function("implementierung_a", |b| {
        b.iter(|| implementierung_a())
    });
    group.bench_function("implementierung_b", |b| {
        b.iter(|| implementierung_b())
    });
    group.finish();
}
```

## Performance-Überwachung

### 1. Metrikerfassung

```rust
use metrics::{counter, gauge, histogram};

fn erfasse_leistung() {
    counter!("api.anfragen.gesamt").increment(1);
    gauge!("system.speicher.nutzung").set(hole_speichernutzung());
    histogram!("api.antwortzeit").record(antwortzeit);
}
```

### 2. Kontinuierliche Überwachung

```rust
fn setup_ueberwachung() {
    metrics_exporter_prometheus::install().expect("Fehler beim Installieren des Prometheus-Exporters");
    
    tokio::spawn(async move {
        erfasse_metriken().await;
    });
}
```

## Weiterführende Literatur

- [Criterion.rs-Dokumentation](https://docs.rs/criterion)
- [Best Practices für Performance-Tests](https://rust-lang.github.io/rust-performance-book/)
- [System-Überwachungsleitfaden](../monitoring/README.md)
- [Benchmarking-Leitfaden](../benchmarking/README.md)
