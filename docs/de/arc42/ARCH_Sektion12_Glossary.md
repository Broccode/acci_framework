# ARCHITECTURE_Section12_Glossary_DE.md

## 12. Glossar

Dieses Dokument liefert Definitionen für zentrale Begriffe und Konzepte, die in der Dokumentation unseres Enterprise Application Frameworks verwendet werden. Das Glossar stellt sicher, dass alle Beteiligten ein gemeinsames Verständnis der verwendeten Terminologie haben.

### Wichtige Begriffe

- **Multi-Tenancy:**  
  Ein Architekturansatz, bei dem mehrere Mandanten auf einer gemeinsamen Plattform betrieben werden, wobei die Daten jeweils isoliert sind.
- **Domain-Driven Design (DDD):**  
  Eine Methodik zur Modellierung komplexer Geschäftsdomänen, die die Kernprozesse und -logiken präzise abbildet.
- **CQRS (Command Query Responsibility Segregation):**  
  Ein Muster, das die Behandlung von Schreib- (Commands) und Leseoperationen (Queries) trennt, um Performance und Skalierbarkeit zu optimieren.
- **Event Sourcing:**  
  Eine Technik, bei der alle Zustandsänderungen der Anwendung als Sequenz von Events gespeichert werden, was vollständige Nachvollziehbarkeit und Wiederherstellung ermöglicht.
- **SBOM (Software Bill of Materials):**  
  Eine detaillierte Auflistung aller Komponenten, Bibliotheken und Abhängigkeiten, die in einem Softwareprodukt enthalten sind.
- **OAuth2/OpenID Connect:**  
  Protokolle und Standards für die sichere Authentifizierung und Autorisierung.
- **API-Versionierung:**  
  Die Praxis, Änderungen an APIs durch die Vergabe von Versionsnummern zu verwalten, um die Rückwärtskompatibilität sicherzustellen.
- **Rate Limiting:**  
  Ein Mechanismus zur Steuerung der Anzahl von API-Anfragen, die ein Client in einem definierten Zeitraum durchführen kann.
- **Circuit Breaker:**  
  Ein Designmuster, das dazu dient, Ausfälle zu erkennen und wiederholte Fehler bei vorübergehenden Problemen zu verhindern.
- **Fallback-Mechanismen:**  
  Strategien, die alternative Verhaltensweisen oder Standardwerte bereitstellen, wenn eine Funktion nicht verfügbar ist.
- **Load Balancing:**  
  Die Verteilung von Netzwerk- oder Anwendungstraffic über mehrere Server zur Optimierung der Ressourcennutzung und Zuverlässigkeit.
- **Zero-Downtime Deployment:**  
  Deployment-Strategien, die es ermöglichen, Systemupdates durchzuführen, ohne den Live-Betrieb zu unterbrechen.
- **Disaster Recovery:**  
  Strategien und Prozesse, um im Falle eines katastrophalen Ausfalls die Systemfunktionalität und Datenintegrität wiederherzustellen.

Dieses Glossar dient als Nachschlagewerk für alle Beteiligten, um ein klares Verständnis der im Framework verwendeten Begriffe zu gewährleisten.
