"""Offline Commercial Sales & License Registry for Daqiq License Generator.

Persists issued license history locally in SQLite:
%LOCALAPPDATA%\\DaqiqLicenseGenerator\\registry.sqlite
Supports search, status filtering, Clone & Renew, CSV/Excel export, and statistics.
"""

from __future__ import annotations

import csv
import datetime
import json
import os
import sqlite3
import sys
from dataclasses import dataclass, asdict
from typing import List, Optional, Dict, Any, Tuple


DEFAULT_APP_DIR_NAME = "DaqiqLicenseGenerator"
DEFAULT_DB_FILENAME = "registry.sqlite"


def get_default_registry_path() -> str:
    """Return platform-specific database path outside the repository."""
    if sys.platform == "win32":
        base = os.environ.get("LOCALAPPDATA") or os.environ.get("APPDATA") or os.path.expanduser("~")
    else:
        base = os.environ.get("XDG_DATA_HOME") or os.path.join(os.path.expanduser("~"), ".local", "share")
    dir_path = os.path.join(base, DEFAULT_APP_DIR_NAME)
    os.makedirs(dir_path, exist_ok=True)
    return os.path.join(dir_path, DEFAULT_DB_FILENAME)


@dataclass
class LicenseRecord:
    """Historical record of an issued license in the local sales ledger."""
    license_id: str
    customer_name: str
    customer_organization: Optional[str]
    customer_phone: Optional[str]
    edition: str
    license_type: str
    issued_at_utc: str
    not_before_utc: str
    expires_at_utc: Optional[str]
    max_devices: int
    device_count: int
    device_fingerprints: List[str]
    device_labels: Dict[str, str]
    features: Dict[str, bool]
    key_id: str
    key_fingerprint: str
    signed_daqlic_content: str
    notes: str = ""
    status: str = "active"  # "active", "renewed", "revoked_note", "expired"
    created_at_utc: str = ""
    updated_at_utc: str = ""
    id: Optional[int] = None

    @property
    def is_expired(self) -> bool:
        if self.license_type == "perpetual":
            return False
        if not self.expires_at_utc:
            return False
        try:
            from .license_builder import parse_iso8601_utc
            exp = parse_iso8601_utc(self.expires_at_utc)
            now = datetime.datetime.now(datetime.timezone.utc)
            return now >= exp
        except Exception:
            return False

    @property
    def is_expiring_soon(self) -> bool:
        """True if subscription expires within 14 days."""
        if self.license_type == "perpetual" or self.is_expired:
            return False
        if not self.expires_at_utc:
            return False
        try:
            from .license_builder import parse_iso8601_utc
            exp = parse_iso8601_utc(self.expires_at_utc)
            now = datetime.datetime.now(datetime.timezone.utc)
            diff = (exp - now).total_seconds()
            return 0 < diff <= (14 * 86400)
        except Exception:
            return False

    @property
    def effective_status(self) -> str:
        if self.status == "revoked_note":
            return "revoked_note"
        if self.status == "renewed":
            return "renewed"
        if self.is_expired:
            return "expired"
        if self.is_expiring_soon:
            return "expiring_soon"
        return "active"


class RegistryDB:
    """SQLite sales and license registry manager."""

    def __init__(self, db_path: Optional[str] = None):
        self.db_path = db_path or get_default_registry_path()
        self._persistent_conn: Optional[sqlite3.Connection] = None
        if self.db_path == ":memory:":
            self._persistent_conn = sqlite3.connect(":memory:")
            self._persistent_conn.row_factory = sqlite3.Row
        self._init_db()

    def _get_connection(self) -> sqlite3.Connection:
        if self._persistent_conn is not None:
            return self._persistent_conn
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        return conn

    def _init_db(self) -> None:
        """Create tables and indexes if they do not exist."""
        if self.db_path != ":memory:":
            os.makedirs(os.path.dirname(os.path.abspath(self.db_path)), exist_ok=True)

        with self._get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS licenses (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    license_id TEXT UNIQUE NOT NULL,
                    customer_name TEXT NOT NULL,
                    customer_organization TEXT,
                    customer_phone TEXT,
                    edition TEXT NOT NULL,
                    license_type TEXT NOT NULL,
                    issued_at_utc TEXT NOT NULL,
                    not_before_utc TEXT NOT NULL,
                    expires_at_utc TEXT,
                    max_devices INTEGER NOT NULL,
                    device_count INTEGER NOT NULL,
                    device_fingerprints_json TEXT NOT NULL,
                    device_labels_json TEXT,
                    features_json TEXT NOT NULL,
                    key_id TEXT NOT NULL,
                    key_fingerprint TEXT NOT NULL,
                    signed_daqlic_content TEXT NOT NULL,
                    notes TEXT,
                    status TEXT NOT NULL DEFAULT 'active',
                    created_at_utc TEXT NOT NULL,
                    updated_at_utc TEXT NOT NULL
                );
            """)
            cursor.execute("CREATE INDEX IF NOT EXISTS idx_licenses_lic_id ON licenses(license_id);")
            cursor.execute("CREATE INDEX IF NOT EXISTS idx_licenses_cust_name ON licenses(customer_name);")
            cursor.execute("CREATE INDEX IF NOT EXISTS idx_licenses_status ON licenses(status);")
            cursor.execute("CREATE INDEX IF NOT EXISTS idx_licenses_expires ON licenses(expires_at_utc);")
            conn.commit()

    def record_license(self, record: LicenseRecord) -> int:
        """Insert or replace an issued license in the registry."""
        now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        created_at = record.created_at_utc or now
        updated_at = now

        fps_json = json.dumps(record.device_fingerprints, ensure_ascii=False)
        labels_json = json.dumps(record.device_labels, ensure_ascii=False)
        features_json = json.dumps(record.features, ensure_ascii=False)

        with self._get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute("""
                INSERT INTO licenses (
                    license_id, customer_name, customer_organization, customer_phone,
                    edition, license_type, issued_at_utc, not_before_utc, expires_at_utc,
                    max_devices, device_count, device_fingerprints_json, device_labels_json,
                    features_json, key_id, key_fingerprint, signed_daqlic_content,
                    notes, status, created_at_utc, updated_at_utc
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(license_id) DO UPDATE SET
                    customer_name=excluded.customer_name,
                    customer_organization=excluded.customer_organization,
                    customer_phone=excluded.customer_phone,
                    edition=excluded.edition,
                    license_type=excluded.license_type,
                    issued_at_utc=excluded.issued_at_utc,
                    not_before_utc=excluded.not_before_utc,
                    expires_at_utc=excluded.expires_at_utc,
                    max_devices=excluded.max_devices,
                    device_count=excluded.device_count,
                    device_fingerprints_json=excluded.device_fingerprints_json,
                    device_labels_json=excluded.device_labels_json,
                    features_json=excluded.features_json,
                    key_id=excluded.key_id,
                    key_fingerprint=excluded.key_fingerprint,
                    signed_daqlic_content=excluded.signed_daqlic_content,
                    notes=excluded.notes,
                    status=excluded.status,
                    updated_at_utc=excluded.updated_at_utc;
            """, (
                record.license_id,
                record.customer_name,
                record.customer_organization,
                record.customer_phone,
                record.edition,
                record.license_type,
                record.issued_at_utc,
                record.not_before_utc,
                record.expires_at_utc,
                record.max_devices,
                record.device_count,
                fps_json,
                labels_json,
                features_json,
                record.key_id,
                record.key_fingerprint,
                record.signed_daqlic_content,
                record.notes,
                record.status,
                created_at,
                updated_at,
            ))
            row_id = cursor.lastrowid
            conn.commit()
            return row_id

    def get_license(self, license_id: str) -> Optional[LicenseRecord]:
        """Fetch a license by its unique license ID."""
        with self._get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM licenses WHERE license_id = ?", (license_id,))
            row = cursor.fetchone()
            if not row:
                return None
            return self._row_to_record(row)

    def search_licenses(
        self,
        query: str = "",
        status_filter: str = "all",
    ) -> List[LicenseRecord]:
        """Search licenses by customer name, phone, license ID, or HWID with status filtering."""
        clean_q = query.strip()
        sql = "SELECT * FROM licenses WHERE 1=1"
        params: List[Any] = []

        if clean_q:
            sql += """ AND (
                license_id LIKE ? OR
                customer_name LIKE ? OR
                customer_phone LIKE ? OR
                customer_organization LIKE ? OR
                device_fingerprints_json LIKE ?
            )"""
            like_param = f"%{clean_q}%"
            params.extend([like_param, like_param, like_param, like_param, like_param])

        sql += " ORDER BY id DESC"

        with self._get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute(sql, params)
            rows = cursor.fetchall()

        records = [self._row_to_record(r) for r in rows]

        if status_filter == "all":
            return records
        elif status_filter == "active":
            return [r for r in records if r.effective_status == "active"]
        elif status_filter == "expiring_soon":
            return [r for r in records if r.effective_status == "expiring_soon"]
        elif status_filter == "expired":
            return [r for r in records if r.effective_status == "expired"]
        elif status_filter == "perpetual":
            return [r for r in records if r.license_type == "perpetual"]
        elif status_filter == "renewed":
            return [r for r in records if r.status == "renewed"]
        elif status_filter == "revoked":
            return [r for r in records if r.status == "revoked_note"]

        return records

    def update_status(self, license_id: str, new_status: str, note: Optional[str] = None) -> bool:
        """Update license status and optional note."""
        now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        with self._get_connection() as conn:
            cursor = conn.cursor()
            if note is not None:
                cursor.execute("""
                    UPDATE licenses SET status = ?, notes = ?, updated_at_utc = ?
                    WHERE license_id = ?
                """, (new_status, note, now, license_id))
            else:
                cursor.execute("""
                    UPDATE licenses SET status = ?, updated_at_utc = ?
                    WHERE license_id = ?
                """, (new_status, now, license_id))
            conn.commit()
            return cursor.rowcount > 0

    def delete_license(self, license_id: str) -> bool:
        """Remove a license record from the ledger."""
        with self._get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute("DELETE FROM licenses WHERE license_id = ?", (license_id,))
            conn.commit()
            return cursor.rowcount > 0

    def get_stats(self) -> Dict[str, int]:
        """Calculate summary statistics for dashboard metric cards."""
        all_records = self.search_licenses(query="", status_filter="all")
        total = len(all_records)
        active = 0
        expiring_soon = 0
        expired = 0
        perpetual = 0

        for r in all_records:
            if r.license_type == "perpetual":
                perpetual += 1
            eff = r.effective_status
            if eff == "active":
                active += 1
            elif eff == "expiring_soon":
                expiring_soon += 1
                active += 1  # Expiring soon is still active!
            elif eff == "expired":
                expired += 1

        return {
            "total": total,
            "active": active,
            "expiring_soon": expiring_soon,
            "expired": expired,
            "perpetual": perpetual,
        }

    def export_to_csv(self, file_path: str) -> None:
        """Export all license records to a UTF-8 CSV with Persian column headers."""
        records = self.search_licenses(query="", status_filter="all")
        headers = [
            "شناسه لایسنس",
            "نام مشتری",
            "سازمان / فروشگاه",
            "شماره تماس",
            "پلن",
            "نوع لایسنس",
            "تاریخ صدور (UTC)",
            "شروع اعتبار (UTC)",
            "تاریخ انقضا (UTC)",
            "سقف دستگاه",
            "تعداد ثبت‌شده",
            "وضعیت",
            "شناسه کلید امضا",
            "یادداشت",
        ]

        with open(file_path, "w", encoding="utf-8-sig", newline="") as f:
            writer = csv.writer(f)
            writer.writerow(headers)
            for r in records:
                writer.writerow([
                    r.license_id,
                    r.customer_name,
                    r.customer_organization or "",
                    r.customer_phone or "",
                    r.edition,
                    "دائمی" if r.license_type == "perpetual" else "اشتراکی",
                    r.issued_at_utc,
                    r.not_before_utc,
                    r.expires_at_utc or "ندارد",
                    r.max_devices,
                    r.device_count,
                    r.effective_status,
                    r.key_id,
                    r.notes or "",
                ])

    def _row_to_record(self, row: sqlite3.Row) -> LicenseRecord:
        fps = json.loads(row["device_fingerprints_json"]) if row["device_fingerprints_json"] else []
        labels = json.loads(row["device_labels_json"]) if row["device_labels_json"] else {}
        features = json.loads(row["features_json"]) if row["features_json"] else {}

        return LicenseRecord(
            id=row["id"],
            license_id=row["license_id"],
            customer_name=row["customer_name"],
            customer_organization=row["customer_organization"],
            customer_phone=row["customer_phone"],
            edition=row["edition"],
            license_type=row["license_type"],
            issued_at_utc=row["issued_at_utc"],
            not_before_utc=row["not_before_utc"],
            expires_at_utc=row["expires_at_utc"],
            max_devices=row["max_devices"],
            device_count=row["device_count"],
            device_fingerprints=fps,
            device_labels=labels,
            features=features,
            key_id=row["key_id"],
            key_fingerprint=row["key_fingerprint"],
            signed_daqlic_content=row["signed_daqlic_content"],
            notes=row["notes"] or "",
            status=row["status"],
            created_at_utc=row["created_at_utc"],
            updated_at_utc=row["updated_at_utc"],
        )
