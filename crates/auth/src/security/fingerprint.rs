use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    Pool, Postgres,
    types::ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network},
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use time::OffsetDateTime;
use tracing::info;
use uuid::Uuid;

use super::config::FingerprintingConfig;
use super::types::RiskLevel;

/// Browser fingerprint data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprint {
    /// User agent string
    pub user_agent: String,
    /// Accept headers
    pub accept_headers: String,
    /// Hash of canvas fingerprint
    pub canvas_hash: Option<String>,
    /// Hash of WebGL fingerprint
    pub webgl_hash: Option<String>,
    /// Available fonts
    pub fonts: Option<Vec<String>>,
    /// Timezone offset in minutes
    pub timezone: Option<i32>,
    /// Screen resolution as (width, height)
    pub screen_resolution: Option<(u32, u32)>,
    /// Color depth
    pub color_depth: Option<u32>,
    /// Browser plugins
    pub plugins: Option<Vec<String>>,
    /// Browser language
    pub language: Option<String>,
    /// Do Not Track setting
    pub do_not_track: Option<bool>,
    /// Cookie enabled flag
    pub cookies_enabled: Option<bool>,
    /// Touch points
    pub touch_points: Option<u32>,
    /// Device memory (GB)
    pub device_memory: Option<f32>,
    /// Hardware concurrency (CPU cores)
    pub hardware_concurrency: Option<u32>,
    /// Platform info
    pub platform: Option<String>,
}

/// Stored fingerprint with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFingerprint {
    /// Internal ID
    pub id: Uuid,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Fingerprint data
    pub fingerprint: BrowserFingerprint,
    /// First seen timestamp
    pub first_seen: DateTime<Utc>,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Last IP address
    pub last_ip: IpAddr,
    /// Session ID
    pub session_id: Option<Uuid>,
    /// Trusted flag (manually verified)
    pub trusted: bool,
}

/// Comparison result between two fingerprints
#[derive(Debug, Clone)]
pub struct FingerprintComparison {
    /// Overall similarity score (0.0 to 1.0)
    pub similarity: f64,
    /// Individual component similarities
    pub component_scores: HashMap<String, f64>,
    /// Risk assessment
    pub risk_level: RiskLevel,
    /// Comparison notes
    pub notes: Vec<String>,
}

/// Repository for storing and retrieving fingerprints
#[async_trait]
pub trait FingerprintRepository: Send + Sync {
    /// Store a new fingerprint
    async fn store_fingerprint(&self, fingerprint: &StoredFingerprint)
    -> Result<(), anyhow::Error>;

    /// Get fingerprints for a user
    async fn get_fingerprints_for_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<StoredFingerprint>, anyhow::Error>;

    /// Update an existing fingerprint
    async fn update_fingerprint(
        &self,
        fingerprint: &StoredFingerprint,
    ) -> Result<(), anyhow::Error>;

    /// Mark a fingerprint as trusted
    async fn mark_as_trusted(&self, id: Uuid, trusted: bool) -> Result<(), anyhow::Error>;

    /// Delete old fingerprints
    async fn delete_old_fingerprints(
        &self,
        tenant_id: Uuid,
        older_than: DateTime<Utc>,
    ) -> Result<u64, anyhow::Error>;
}

/// PostgreSQL implementation of fingerprint repository
pub struct PostgresFingerprintRepository {
    pool: Pool<Postgres>,
}

impl PostgresFingerprintRepository {
    /// Create a new PostgreSQL fingerprint repository
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Convert OffsetDateTime to chrono::DateTime<Utc>
    fn offset_to_chrono_utc(offset: OffsetDateTime) -> DateTime<Utc> {
        let timestamp = offset.unix_timestamp();
        let nanos = offset.nanosecond();

        DateTime::from_timestamp(timestamp, nanos).unwrap_or_else(|| {
            DateTime::from_timestamp(0, 0).expect("Failed to create fallback timestamp")
        })
    }

    /// Convert chrono::DateTime<Utc> to OffsetDateTime
    fn chrono_utc_to_offset(dt: DateTime<Utc>) -> OffsetDateTime {
        let timestamp = dt.timestamp();
        let nanos = dt.timestamp_subsec_nanos();
        match OffsetDateTime::from_unix_timestamp(timestamp) {
            Ok(odt) => match odt.replace_nanosecond(nanos) {
                Ok(result) => result,
                Err(_) => OffsetDateTime::from_unix_timestamp(0)
                    .expect("Failed to create Unix timestamp from 0"),
            },
            Err(_) => OffsetDateTime::from_unix_timestamp(0)
                .expect("Failed to create Unix timestamp from 0"),
        }
    }

    /// Convert IpAddr to IpNetwork
    fn ip_addr_to_network(addr: IpAddr) -> IpNetwork {
        match addr {
            IpAddr::V4(ipv4) => IpNetwork::new(IpAddr::V4(ipv4), 32).unwrap_or_else(|_| {
                IpNetwork::V4(
                    Ipv4Network::new(std::net::Ipv4Addr::new(0, 0, 0, 0), 32)
                        .expect("Failed to create fallback IPv4 network"),
                )
            }),
            IpAddr::V6(ipv6) => IpNetwork::new(IpAddr::V6(ipv6), 128).unwrap_or_else(|_| {
                IpNetwork::V6(
                    Ipv6Network::new(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 128)
                        .expect("Failed to create IPv6 network"),
                )
            }),
        }
    }
}

#[async_trait]
impl FingerprintRepository for PostgresFingerprintRepository {
    async fn store_fingerprint(
        &self,
        fingerprint: &StoredFingerprint,
    ) -> Result<(), anyhow::Error> {
        let fingerprint_json = serde_json::to_value(&fingerprint.fingerprint)?;
        let first_seen = Self::chrono_utc_to_offset(fingerprint.first_seen);
        let last_seen = Self::chrono_utc_to_offset(fingerprint.last_seen);
        let last_ip = Self::ip_addr_to_network(fingerprint.last_ip);

        sqlx::query!(
            r#"
            INSERT INTO fingerprints (
                id, tenant_id, user_id, fingerprint, first_seen, last_seen, 
                last_ip, session_id, trusted
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            fingerprint.id,
            fingerprint.tenant_id,
            fingerprint.user_id,
            fingerprint_json,
            first_seen,
            last_seen,
            last_ip,
            fingerprint.session_id,
            fingerprint.trusted
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_fingerprints_for_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<StoredFingerprint>, anyhow::Error> {
        let records = sqlx::query!(
            r#"
            SELECT id, tenant_id, user_id, fingerprint, first_seen, last_seen, 
                   last_ip, session_id, trusted
            FROM fingerprints
            WHERE tenant_id = $1 AND user_id = $2
            ORDER BY last_seen DESC
            "#,
            tenant_id,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut fingerprints = Vec::new();

        for record in records {
            let fingerprint_data: BrowserFingerprint = serde_json::from_value(record.fingerprint)?;

            // Extract IP address from network type
            let ip_addr = {
                let ip_str = record.last_ip.to_string();
                let ip_parts: Vec<&str> = ip_str.split('/').collect();
                ip_parts[0].parse::<IpAddr>().unwrap_or_else(|_| {
                    "0.0.0.0"
                        .parse()
                        .expect("Failed to parse fallback IP address")
                })
            };

            fingerprints.push(StoredFingerprint {
                id: record.id,
                tenant_id: record.tenant_id,
                user_id: record.user_id,
                fingerprint: fingerprint_data,
                first_seen: Self::offset_to_chrono_utc(record.first_seen),
                last_seen: Self::offset_to_chrono_utc(record.last_seen),
                last_ip: ip_addr,
                session_id: record.session_id,
                trusted: record.trusted,
            });
        }

        Ok(fingerprints)
    }

    async fn update_fingerprint(
        &self,
        fingerprint: &StoredFingerprint,
    ) -> Result<(), anyhow::Error> {
        let fingerprint_json = serde_json::to_value(&fingerprint.fingerprint)?;
        let last_seen = Self::chrono_utc_to_offset(fingerprint.last_seen);
        let last_ip = Self::ip_addr_to_network(fingerprint.last_ip);

        sqlx::query!(
            r#"
            UPDATE fingerprints
            SET fingerprint = $1, last_seen = $2, last_ip = $3, session_id = $4, trusted = $5
            WHERE id = $6 AND tenant_id = $7 AND user_id = $8
            "#,
            fingerprint_json,
            last_seen,
            last_ip,
            fingerprint.session_id,
            fingerprint.trusted,
            fingerprint.id,
            fingerprint.tenant_id,
            fingerprint.user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_as_trusted(&self, id: Uuid, trusted: bool) -> Result<(), anyhow::Error> {
        sqlx::query!(
            r#"
            UPDATE fingerprints
            SET trusted = $1
            WHERE id = $2
            "#,
            trusted,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_old_fingerprints(
        &self,
        tenant_id: Uuid,
        older_than: DateTime<Utc>,
    ) -> Result<u64, anyhow::Error> {
        let offset_time = Self::chrono_utc_to_offset(older_than);

        let result = sqlx::query!(
            r#"
            DELETE FROM fingerprints
            WHERE tenant_id = $1 AND last_seen < $2
            "#,
            tenant_id,
            offset_time
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Service for managing and comparing fingerprints
pub struct FingerprintService {
    repository: Arc<dyn FingerprintRepository>,
    config: FingerprintingConfig,
}

impl FingerprintService {
    /// Create a new fingerprint service
    pub fn new(repository: Arc<dyn FingerprintRepository>, config: FingerprintingConfig) -> Self {
        Self { repository, config }
    }

    /// Store a fingerprint for a user
    pub async fn store_fingerprint(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        fingerprint: &BrowserFingerprint,
        ip_address: &str,
        session_id: Option<Uuid>,
    ) -> Result<Uuid, anyhow::Error> {
        // Check if we already have a similar fingerprint for this user
        let existing = self
            .repository
            .get_fingerprints_for_user(tenant_id, user_id)
            .await?;

        for stored in &existing {
            let comparison = self.compare_fingerprints(&stored.fingerprint, fingerprint);

            // If very similar, update the existing one
            if comparison.similarity >= self.config.similarity_threshold as f64 {
                let mut updated = stored.clone();
                updated.last_seen = Utc::now();

                // Parse the IP address
                let ip_addr = ip_address.parse::<IpAddr>().unwrap_or_else(|_| {
                    "0.0.0.0"
                        .parse()
                        .expect("Failed to parse fallback IP address")
                });
                updated.last_ip = ip_addr;

                updated.session_id = session_id;

                self.repository.update_fingerprint(&updated).await?;
                return Ok(updated.id);
            }
        }

        // Otherwise, store as a new fingerprint
        let now = Utc::now();
        let fp_id = Uuid::new_v4();

        // Parse the IP address
        let ip_addr = ip_address.parse::<IpAddr>().unwrap_or_else(|_| {
            "0.0.0.0"
                .parse()
                .expect("Failed to parse fallback IP address")
        });

        let stored = StoredFingerprint {
            id: fp_id,
            tenant_id,
            user_id,
            fingerprint: fingerprint.clone(),
            first_seen: now,
            last_seen: now,
            last_ip: ip_addr,
            session_id,
            trusted: false, // New fingerprints start untrusted
        };

        self.repository.store_fingerprint(&stored).await?;

        Ok(fp_id)
    }

    /// Verify a fingerprint against known user fingerprints
    pub async fn verify_fingerprint(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        fingerprint: &BrowserFingerprint,
    ) -> Result<(RiskLevel, Option<String>), anyhow::Error> {
        if !self.config.enabled {
            return Ok((RiskLevel::Low, None));
        }

        let existing = self
            .repository
            .get_fingerprints_for_user(tenant_id, user_id)
            .await?;

        if existing.is_empty() {
            // First time we've seen this user, no risk assessment possible
            return Ok((
                RiskLevel::Low,
                Some("First fingerprint for user".to_string()),
            ));
        }

        let mut best_similarity = 0.0;
        let mut best_match: Option<&StoredFingerprint> = None;
        let mut detailed_notes = Vec::new();

        for stored in &existing {
            let comparison = self.compare_fingerprints(&stored.fingerprint, fingerprint);
            detailed_notes.extend(comparison.notes.clone());

            if comparison.similarity > best_similarity {
                best_similarity = comparison.similarity;
                best_match = Some(stored);
            }
        }

        // Risk assessment based on similarity threshold
        let risk_level = if best_similarity >= self.config.similarity_threshold as f64 {
            if let Some(matched) = best_match {
                if matched.trusted {
                    RiskLevel::Low
                } else {
                    RiskLevel::Medium
                }
            } else {
                RiskLevel::Medium
            }
        } else if best_similarity >= (self.config.similarity_threshold as f64 * 0.8) {
            RiskLevel::Medium
        } else if best_similarity >= (self.config.similarity_threshold as f64 * 0.6) {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        };

        let note = format!(
            "Best similarity: {:.2}%, threshold: {:.2}%, risk: {:?}",
            best_similarity * 100.0,
            self.config.similarity_threshold as f64 * 100.0,
            risk_level
        );

        detailed_notes.push(note.clone());

        Ok((risk_level, Some(detailed_notes.join("; "))))
    }

    /// Compare two fingerprints
    pub fn compare_fingerprints(
        &self,
        fp1: &BrowserFingerprint,
        fp2: &BrowserFingerprint,
    ) -> FingerprintComparison {
        use std::collections::HashMap;

        let mut scores = HashMap::new();
        let mut notes = Vec::new();

        // User agent comparison
        let ua_similarity = string_similarity(&fp1.user_agent, &fp2.user_agent);
        scores.insert("user_agent".to_string(), ua_similarity);

        if ua_similarity < 0.8 {
            notes.push(format!(
                "User agent changed: {:.2}% similar",
                ua_similarity * 100.0
            ));
        }

        // Accept headers
        let accept_similarity = string_similarity(&fp1.accept_headers, &fp2.accept_headers);
        scores.insert("accept_headers".to_string(), accept_similarity);

        // Canvas hash (if configured and available)
        if self.config.collect_canvas {
            match (&fp1.canvas_hash, &fp2.canvas_hash) {
                (Some(h1), Some(h2)) => {
                    let canvas_match = h1 == h2;
                    scores.insert(
                        "canvas_hash".to_string(),
                        if canvas_match { 1.0 } else { 0.0 },
                    );

                    if !canvas_match {
                        notes.push("Canvas fingerprint changed".to_string());
                    }
                },
                _ => {
                    // One or both missing, partial score
                    scores.insert("canvas_hash".to_string(), 0.5);
                },
            }
        }

        // WebGL hash (if configured and available)
        if self.config.collect_webgl {
            match (&fp1.webgl_hash, &fp2.webgl_hash) {
                (Some(h1), Some(h2)) => {
                    let webgl_match = h1 == h2;
                    scores.insert(
                        "webgl_hash".to_string(),
                        if webgl_match { 1.0 } else { 0.0 },
                    );

                    if !webgl_match {
                        notes.push("WebGL fingerprint changed".to_string());
                    }
                },
                _ => {
                    // One or both missing, partial score
                    scores.insert("webgl_hash".to_string(), 0.5);
                },
            }
        }

        // Fonts (if configured and available)
        if self.config.collect_fonts {
            match (&fp1.fonts, &fp2.fonts) {
                (Some(f1), Some(f2)) => {
                    let fonts_similarity = set_similarity(f1, f2);
                    scores.insert("fonts".to_string(), fonts_similarity);

                    if fonts_similarity < 0.8 {
                        notes.push(format!(
                            "Font list changed: {:.2}% similar",
                            fonts_similarity * 100.0
                        ));
                    }
                },
                _ => {
                    // One or both missing, partial score
                    scores.insert("fonts".to_string(), 0.5);
                },
            }
        }

        // Screen resolution
        match (&fp1.screen_resolution, &fp2.screen_resolution) {
            (Some(r1), Some(r2)) => {
                let res_match = r1 == r2;
                scores.insert(
                    "screen_resolution".to_string(),
                    if res_match { 1.0 } else { 0.0 },
                );

                if !res_match {
                    notes.push(format!("Screen resolution changed: {:?} -> {:?}", r1, r2));
                }
            },
            _ => {
                // One or both missing, partial score
                scores.insert("screen_resolution".to_string(), 0.5);
            },
        }

        // Platform info
        match (&fp1.platform, &fp2.platform) {
            (Some(p1), Some(p2)) => {
                let platform_match = p1 == p2;
                scores.insert(
                    "platform".to_string(),
                    if platform_match { 1.0 } else { 0.0 },
                );

                if !platform_match {
                    notes.push(format!("Platform changed: {} -> {}", p1, p2));
                }
            },
            _ => {
                scores.insert("platform".to_string(), 0.5);
            },
        }

        // Calculate overall score
        let mut total_score = 0.0;
        let mut weights = 0.0;

        // User agent is most important
        total_score += scores.get("user_agent").unwrap_or(&0.5) * 3.0;
        weights += 3.0;

        // Other components weighted by importance
        for (key, score) in &scores {
            if key != "user_agent" {
                // Each component gets weight 1.0 except user agent
                total_score += score;
                weights += 1.0;
            }
        }

        let overall_similarity = total_score / weights;

        // Determine risk level
        let risk_level = if overall_similarity >= self.config.similarity_threshold as f64 {
            RiskLevel::Low
        } else if overall_similarity >= (self.config.similarity_threshold as f64 * 0.8) {
            RiskLevel::Medium
        } else if overall_similarity >= (self.config.similarity_threshold as f64 * 0.6) {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        };

        FingerprintComparison {
            similarity: overall_similarity,
            component_scores: scores,
            risk_level,
            notes,
        }
    }

    /// Clean up old fingerprints based on retention policy
    pub async fn cleanup_old_fingerprints(&self, tenant_id: Uuid) -> Result<u64, anyhow::Error> {
        let retention_days = self.config.retention_days;
        let cutoff = Utc::now() - Duration::days(retention_days as i64);

        let deleted = self
            .repository
            .delete_old_fingerprints(tenant_id, cutoff)
            .await?;

        if deleted > 0 {
            info!(
                "Deleted {} old fingerprints for tenant {}",
                deleted, tenant_id
            );
        }

        Ok(deleted)
    }
}

/// Calculate string similarity score (0.0 to 1.0)
fn string_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }

    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let distance = levenshtein_distance(s1, s2);
    let max_len = std::cmp::max(len1, len2);

    1.0 - (distance as f64 / max_len as f64)
}

/// Calculate set similarity (Jaccard index)
fn set_similarity<T: Eq + std::hash::Hash>(set1: &[T], set2: &[T]) -> f64 {
    use std::collections::HashSet;

    let s1: HashSet<&T> = set1.iter().collect();
    let s2: HashSet<&T> = set2.iter().collect();

    let intersection = s1.intersection(&s2).count();
    let union = s1.len() + s2.len() - intersection;

    if union == 0 {
        return 1.0; // Both sets are empty
    }

    intersection as f64 / union as f64
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();

    let len1 = v1.len();
    let len2 = v2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }

    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::types::DeviceFingerprint;

    #[test]
    fn test_string_similarity() {
        // Test exact match
        assert_eq!(string_similarity("hello", "hello"), 1.0);

        // Test similar strings
        assert!(string_similarity("hello", "hallo") > 0.6);
        assert!(string_similarity("hello", "helo") >= 0.8);
        assert!(string_similarity("testing", "tasting") > 0.7);

        // Test different strings
        assert!(string_similarity("completely", "different") < 0.5);
        assert!(string_similarity("apple", "orange") < 0.5);

        // Test edge cases
        assert_eq!(string_similarity("", ""), 1.0); // Empty strings are identical
        assert_eq!(string_similarity("", "something"), 0.0); // Empty vs non-empty
    }

    #[test]
    fn test_set_similarity() {
        // Test partially overlapping sets
        let set1 = vec!["a", "b", "c"];
        let set2 = vec!["b", "c", "d"];
        assert_eq!(set_similarity(&set1, &set2), 0.5);

        // Test disjoint sets
        let set3 = vec!["x", "y", "z"];
        assert_eq!(set_similarity(&set1, &set3), 0.0);

        // Test identical sets
        let set4 = vec!["a", "b", "c"];
        assert_eq!(set_similarity(&set1, &set4), 1.0);

        // Test empty sets
        let empty: Vec<&str> = vec![];
        assert_eq!(set_similarity(&empty, &empty), 1.0); // Both empty = identical
        assert_eq!(set_similarity(&empty, &set1), 0.0); // Empty vs non-empty = 0 similarity
    }

    #[test]
    fn test_levenshtein_distance() {
        // Test identical strings
        assert_eq!(levenshtein_distance("hello", "hello"), 0);

        // Test simple substitutions
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
        assert_eq!(levenshtein_distance("hello", "helLo"), 1);

        // Test insertions
        assert_eq!(levenshtein_distance("hello", "heello"), 1);
        assert_eq!(levenshtein_distance("hello", "hellox"), 1);

        // Test deletions
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("hello", "hell"), 1);

        // Test mixed operations
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);

        // Test edge cases
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "xyz"), 3);
    }

    #[test]
    fn test_fingerprint_creation() {
        // Test fingerprint creation and builder pattern
        let fp = DeviceFingerprint::new("ua_hash".to_string())
            .with_browser("Firefox".to_string())
            .with_platform("Linux".to_string())
            .with_screen_resolution("1920x1080".to_string())
            .with_language("en-US".to_string());

        assert_eq!(fp.user_agent_hash, "ua_hash");
        assert_eq!(fp.browser.as_deref(), Some("Firefox"));
        assert_eq!(fp.platform.as_deref(), Some("Linux"));
        assert_eq!(fp.screen_resolution.as_deref(), Some("1920x1080"));
        assert_eq!(fp.language.as_deref(), Some("en-US"));
        assert_eq!(fp.color_depth, None);
        assert_eq!(fp.do_not_track, None);
    }

    #[test]
    fn test_fingerprint_comparison() {
        // Create base fingerprint
        let fp1 = DeviceFingerprint::new("hash1".to_string())
            .with_browser("Firefox".to_string())
            .with_platform("Linux".to_string())
            .with_screen_resolution("1920x1080".to_string())
            .with_language("en-US".to_string())
            .with_timezone("UTC".to_string());

        // Create similar fingerprint (just browser changed)
        let fp2 = DeviceFingerprint::new("hash2".to_string())
            .with_browser("Chrome".to_string())
            .with_platform("Linux".to_string())
            .with_screen_resolution("1920x1080".to_string())
            .with_language("en-US".to_string())
            .with_timezone("UTC".to_string());

        // Create different fingerprint
        let fp3 = DeviceFingerprint::new("hash3".to_string())
            .with_browser("Safari".to_string())
            .with_platform("MacOS".to_string())
            .with_screen_resolution("1440x900".to_string())
            .with_language("fr-FR".to_string())
            .with_timezone("Europe/Paris".to_string());

        // Compare fingerprints
        let comparison_score = compare_fingerprints(&fp1, &fp2);
        assert!(
            comparison_score > 0.7,
            "Similar fingerprints should have high similarity score"
        );

        let comparison_score = compare_fingerprints(&fp1, &fp3);
        assert!(
            comparison_score < 0.5,
            "Different fingerprints should have low similarity score"
        );
    }

    // Helper function for unit tests (simplified version of fingerprint comparison)
    fn compare_fingerprints(fp1: &DeviceFingerprint, fp2: &DeviceFingerprint) -> f64 {
        let mut scores = Vec::new();
        let mut weights = Vec::new();

        // Compare platform
        if let (Some(p1), Some(p2)) = (&fp1.platform, &fp2.platform) {
            scores.push(string_similarity(p1, p2));
            weights.push(0.2);
        }

        // Compare browser
        if let (Some(b1), Some(b2)) = (&fp1.browser, &fp2.browser) {
            scores.push(string_similarity(b1, b2));
            weights.push(0.2);
        }

        // Compare screen resolution
        if let (Some(r1), Some(r2)) = (&fp1.screen_resolution, &fp2.screen_resolution) {
            scores.push(string_similarity(r1, r2));
            weights.push(0.1);
        }

        // Compare language
        if let (Some(l1), Some(l2)) = (&fp1.language, &fp2.language) {
            scores.push(string_similarity(l1, l2));
            weights.push(0.15);
        }

        // Compare timezone
        if let (Some(t1), Some(t2)) = (&fp1.timezone, &fp2.timezone) {
            scores.push(string_similarity(t1, t2));
            weights.push(0.2);
        }

        // If no comparable fields, return 0.0
        if scores.is_empty() {
            return 0.0;
        }

        // Calculate weighted average
        let total_weight: f64 = weights.iter().sum();
        let mut weighted_sum = 0.0;

        for i in 0..scores.len() {
            weighted_sum += scores[i] * weights[i];
        }

        weighted_sum / total_weight
    }
}
