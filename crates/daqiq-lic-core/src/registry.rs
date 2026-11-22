use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::error::LicError;
use crate::schema::LicenseEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryRecord {
    pub id: i64,
    pub license_id: String,
    pub key_id: String,
    pub customer_name: String,
    pub customer_org: Option<String>,
    pub customer_contact: Option<String>,
    pub license_type: String,
    pub issued_at: String,
    pub not_before: String,
    pub expires_at: Option<String>,
    pub max_devices: u32,
    pub device_fingerprints: String, // newline or comma-separated
    pub features_json: String,
    pub full_envelope_json: String,
    pub created_at: String,
}

#[cfg(feature = "sqlite")]
pub struct RegistryDb {
    conn: rusqlite::Connection,
}

#[cfg(feature = "sqlite")]
impl RegistryDb {
    pub fn open(path: &Path) -> Result<Self, LicError> {
        let conn = rusqlite::Connection::open(path)?;
        let reg = Self { conn };
        reg.init_tables()?;
        Ok(reg)
    }

    pub fn open_in_memory() -> Result<Self, LicError> {
        let conn = rusqlite::Connection::open_in_memory()?;
        let reg = Self { conn };
        reg.init_tables()?;
        Ok(reg)
    }

    fn init_tables(&self) -> Result<(), LicError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS licenses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                license_id TEXT UNIQUE NOT NULL,
                key_id TEXT NOT NULL,
                customer_name TEXT NOT NULL,
                customer_org TEXT,
                customer_contact TEXT,
                license_type TEXT NOT NULL,
                issued_at TEXT NOT NULL,
                not_before TEXT NOT NULL,
                expires_at TEXT,
                max_devices INTEGER NOT NULL,
                device_fingerprints TEXT NOT NULL,
                features_json TEXT NOT NULL,
                full_envelope_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_licenses_license_id ON licenses(license_id);
            CREATE INDEX IF NOT EXISTS idx_licenses_customer_name ON licenses(customer_name);
            CREATE INDEX IF NOT EXISTS idx_licenses_created_at ON licenses(created_at);
            "#,
        )?;
        Ok(())
    }

    pub fn insert_license(&self, envelope: &LicenseEnvelope) -> Result<(), LicError> {
        use rusqlite::params;
        let p = &envelope.payload;
        let c = &p.customer;
        let now_utc = chrono::Utc::now().to_rfc3339();
        let fingerprints = p.device_fingerprints.join("\n");
        let features_json = serde_json::to_string(&p.features)
            .map_err(|e| LicError::Serialization(e.to_string()))?;
        let full_json = serde_json::to_string_pretty(envelope)
            .map_err(|e| LicError::Serialization(e.to_string()))?;

        self.conn.execute(
            r#"
            INSERT INTO licenses (
                license_id, key_id, customer_name, customer_org, customer_contact,
                license_type, issued_at, not_before, expires_at, max_devices,
                device_fingerprints, features_json, full_envelope_json, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(license_id) DO UPDATE SET
                key_id = excluded.key_id,
                customer_name = excluded.customer_name,
                customer_org = excluded.customer_org,
                customer_contact = excluded.customer_contact,
                license_type = excluded.license_type,
                issued_at = excluded.issued_at,
                not_before = excluded.not_before,
                expires_at = excluded.expires_at,
                max_devices = excluded.max_devices,
                device_fingerprints = excluded.device_fingerprints,
                features_json = excluded.features_json,
                full_envelope_json = excluded.full_envelope_json
            "#,
            params![
                p.license_id,
                p.key_id,
                c.name,
                c.organization,
                c.contact,
                p.license_type,
                p.issued_at,
                p.not_before,
                p.expires_at,
                p.max_devices,
                fingerprints,
                features_json,
                full_json,
                now_utc,
            ],
        )?;

        Ok(())
    }

    pub fn list_licenses(&self, limit: u32, offset: u32) -> Result<Vec<RegistryRecord>, LicError> {
        use rusqlite::params;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, license_id, key_id, customer_name, customer_org, customer_contact,
                   license_type, issued_at, not_before, expires_at, max_devices,
                   device_fingerprints, features_json, full_envelope_json, created_at
            FROM licenses
            ORDER BY id DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok(RegistryRecord {
                id: row.get(0)?,
                license_id: row.get(1)?,
                key_id: row.get(2)?,
                customer_name: row.get(3)?,
                customer_org: row.get(4)?,
                customer_contact: row.get(5)?,
                license_type: row.get(6)?,
                issued_at: row.get(7)?,
                not_before: row.get(8)?,
                expires_at: row.get(9)?,
                max_devices: row.get(10)?,
                device_fingerprints: row.get(11)?,
                features_json: row.get(12)?,
                full_envelope_json: row.get(13)?,
                created_at: row.get(14)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn search_licenses(&self, query: &str) -> Result<Vec<RegistryRecord>, LicError> {
        use rusqlite::params;
        let pattern = format!("%{}%", query.trim());
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, license_id, key_id, customer_name, customer_org, customer_contact,
                   license_type, issued_at, not_before, expires_at, max_devices,
                   device_fingerprints, features_json, full_envelope_json, created_at
            FROM licenses
            WHERE license_id LIKE ?1
               OR customer_name LIKE ?1
               OR customer_org LIKE ?1
               OR customer_contact LIKE ?1
               OR device_fingerprints LIKE ?1
            ORDER BY id DESC
            LIMIT 100
            "#,
        )?;

        let rows = stmt.query_map(params![pattern], |row| {
            Ok(RegistryRecord {
                id: row.get(0)?,
                license_id: row.get(1)?,
                key_id: row.get(2)?,
                customer_name: row.get(3)?,
                customer_org: row.get(4)?,
                customer_contact: row.get(5)?,
                license_type: row.get(6)?,
                issued_at: row.get(7)?,
                not_before: row.get(8)?,
                expires_at: row.get(9)?,
                max_devices: row.get(10)?,
                device_fingerprints: row.get(11)?,
                features_json: row.get(12)?,
                full_envelope_json: row.get(13)?,
                created_at: row.get(14)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn export_csv(&self) -> Result<String, LicError> {
        let records = self.list_licenses(10_000, 0)?;
        format_csv_records(&records)
    }
}

#[cfg(not(feature = "sqlite"))]
pub struct RegistryDb {
    records: std::sync::Mutex<Vec<RegistryRecord>>,
}

#[cfg(not(feature = "sqlite"))]
impl RegistryDb {
    pub fn open(_path: &Path) -> Result<Self, LicError> {
        Ok(Self {
            records: std::sync::Mutex::new(Vec::new()),
        })
    }

    pub fn open_in_memory() -> Result<Self, LicError> {
        Ok(Self {
            records: std::sync::Mutex::new(Vec::new()),
        })
    }

    pub fn insert_license(&self, envelope: &LicenseEnvelope) -> Result<(), LicError> {
        let p = &envelope.payload;
        let c = &p.customer;
        let now_utc = chrono::Utc::now().to_rfc3339();
        let fingerprints = p.device_fingerprints.join("\n");
        let features_json = serde_json::to_string(&p.features)
            .map_err(|e| LicError::Serialization(e.to_string()))?;
        let full_json = serde_json::to_string_pretty(envelope)
            .map_err(|e| LicError::Serialization(e.to_string()))?;

        let mut lock = self.records.lock().unwrap();
        let new_id = (lock.len() + 1) as i64;

        if let Some(pos) = lock.iter().position(|r| r.license_id == p.license_id) {
            lock[pos] = RegistryRecord {
                id: lock[pos].id,
                license_id: p.license_id.clone(),
                key_id: p.key_id.clone(),
                customer_name: c.name.clone(),
                customer_org: c.organization.clone(),
                customer_contact: c.contact.clone(),
                license_type: p.license_type.clone(),
                issued_at: p.issued_at.clone(),
                not_before: p.not_before.clone(),
                expires_at: p.expires_at.clone(),
                max_devices: p.max_devices,
                device_fingerprints: fingerprints,
                features_json,
                full_envelope_json: full_json,
                created_at: now_utc,
            };
        } else {
            lock.push(RegistryRecord {
                id: new_id,
                license_id: p.license_id.clone(),
                key_id: p.key_id.clone(),
                customer_name: c.name.clone(),
                customer_org: c.organization.clone(),
                customer_contact: c.contact.clone(),
                license_type: p.license_type.clone(),
                issued_at: p.issued_at.clone(),
                not_before: p.not_before.clone(),
                expires_at: p.expires_at.clone(),
                max_devices: p.max_devices,
                device_fingerprints: fingerprints,
                features_json,
                full_envelope_json: full_json,
                created_at: now_utc,
            });
        }
        Ok(())
    }

    pub fn list_licenses(&self, limit: u32, offset: u32) -> Result<Vec<RegistryRecord>, LicError> {
        let lock = self.records.lock().unwrap();
        let start = offset as usize;
        let end = (start + limit as usize).min(lock.len());
        if start >= lock.len() {
            return Ok(Vec::new());
        }
        let slice = &lock[start..end];
        let mut res = slice.to_vec();
        res.reverse();
        Ok(res)
    }

    pub fn search_licenses(&self, query: &str) -> Result<Vec<RegistryRecord>, LicError> {
        let lock = self.records.lock().unwrap();
        let q = query.trim().to_lowercase();
        let mut matches = Vec::new();
        for r in lock.iter() {
            if r.license_id.to_lowercase().contains(&q)
                || r.customer_name.to_lowercase().contains(&q)
                || r.customer_org.as_deref().unwrap_or("").to_lowercase().contains(&q)
                || r.customer_contact.as_deref().unwrap_or("").to_lowercase().contains(&q)
                || r.device_fingerprints.to_lowercase().contains(&q)
            {
                matches.push(r.clone());
            }
        }
        matches.reverse();
        Ok(matches)
    }

    pub fn export_csv(&self) -> Result<String, LicError> {
        let lock = self.records.lock().unwrap();
        format_csv_records(&lock)
    }
}

fn format_csv_records(records: &[RegistryRecord]) -> Result<String, LicError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record([
        "ID",
        "License ID",
        "Key ID",
        "Customer Name",
        "Organization",
        "Contact",
        "Type",
        "Issued At",
        "Not Before",
        "Expires At",
        "Max Devices",
        "Fingerprints",
        "Created At",
    ])
    .map_err(|e| LicError::Serialization(e.to_string()))?;

    for r in records {
        wtr.write_record(&[
            r.id.to_string(),
            r.license_id.clone(),
            r.key_id.clone(),
            r.customer_name.clone(),
            r.customer_org.clone().unwrap_or_default(),
            r.customer_contact.clone().unwrap_or_default(),
            r.license_type.clone(),
            r.issued_at.clone(),
            r.not_before.clone(),
            r.expires_at.clone().unwrap_or_default(),
            r.max_devices.to_string(),
            r.device_fingerprints.replace('\n', "; "),
            r.created_at.clone(),
        ])
        .map_err(|e| LicError::Serialization(e.to_string()))?;
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| LicError::Serialization(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| LicError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{CustomerInfo, LicensePayload};
    use std::collections::BTreeMap;

    #[test]
    fn test_registry_operations() {
        let db = RegistryDb::open_in_memory().unwrap();
        let payload = LicensePayload {
            customer: CustomerInfo {
                name: "فروشگاه نمونه".to_string(),
                organization: None,
                contact: Some("+989123456789".to_string()),
            },
            device_fingerprints: vec!["fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e".to_string()],
            expires_at: None,
            features: BTreeMap::new(),
            format_version: 1,
            issued_at: "2026-11-22T00:00:00Z".to_string(),
            key_id: "daqiq-prod-v1".to_string(),
            license_id: "DQL-1405-001".to_string(),
            license_type: "perpetual".to_string(),
            max_app_version: None,
            max_devices: 1,
            min_app_version: Some("4.9.0".to_string()),
            not_before: "2026-11-22T00:00:00Z".to_string(),
            product_id: "daqiq.desktop".to_string(),
        };

        let env = LicenseEnvelope {
            algorithm: "Ed25519".to_string(),
            envelope_version: 1,
            key_id: "daqiq-prod-v1".to_string(),
            payload,
            signature: "dummy_signature".to_string(),
        };

        db.insert_license(&env).unwrap();

        let list = db.list_licenses(10, 0).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].license_id, "DQL-1405-001");
        assert_eq!(list[0].customer_name, "فروشگاه نمونه");

        let search = db.search_licenses("نمونه").unwrap();
        assert_eq!(search.len(), 1);

        let csv = db.export_csv().unwrap();
        assert!(csv.contains("DQL-1405-001"));
        assert!(csv.contains("فروشگاه نمونه"));
    }
}
