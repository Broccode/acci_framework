# ARCHITECTURE_Section5_Building_Block_View_DE.md

## 5. Bausteinsicht

Dieses Dokument bietet einen detaillierten Überblick über die Hauptkomponenten und Container, die die Struktur unseres Enterprise Application Frameworks bilden. Es beschreibt, wie das System in modulare Bausteine unterteilt ist.

### Container-Übersicht

- **API-Container:**  
  Beinhaltet Authentifizierungs- und Autorisierungsdienste sowie das API-Gateway, das Anfragen weiterleitet.
- **Business-Logik-Container:**  
  Implementiert zentrale Funktionen unter Einsatz von Domain-Driven Design, Event Sourcing und CQRS.
- **Datenbank-Container:**  
  Verwendet PostgreSQL zur persistenten Datenspeicherung, unterstützt durch Redis als Caching-Schicht zur Leistungssteigerung.
- **Integrations-Container:**  
  Verwaltert die Schnittstellen zu externen Systemen wie SMTP, HR-/Verzeichnisdiensten und Monitoring-Tools.

### Wichtige Komponenten

- **User Management Modul:**  
  Verantwortlich für die Benutzerregistrierung, Authentifizierung (inklusive Multi-Faktor und SSO) sowie die Verwaltung von Benutzerprofilen.
- **Lizenzmanagement Modul:**  
  Steuert die Verwaltung von Lizenztokens, deren Validierung und Überwachung, inklusive Funktionen wie Offline-Validierung und Notfall-Overrides.
- **Internationalisierungsmodul:**  
  Unterstützt die dynamische Sprachumschaltung und kulturelle Formatierung.
- **Plugin- & Workflow-Engine:**  
  Ermöglicht die Erweiterung der Geschäftslogik durch Plugins und die Integration von Workflow-Engines.

### Vorteile

- **Modularität:**  
  Jede Komponente ist entkoppelt, was Wartung und Erweiterungen erleichtert.
- **Wiederverwendbarkeit:**  
  Gut definierte Module können in verschiedenen Anwendungen wiederverwendet werden.
- **Klare Schnittstellen:**  
  Standardisierte APIs gewährleisten eine nahtlose Kommunikation zwischen den Komponenten und mit externen Systemen.

Diese Bausteinsicht bildet den strukturellen Bauplan unseres Frameworks.
