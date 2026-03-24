#![allow(dead_code)]
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use chrono::Utc;

/// Alasan pembekuan akun
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FreezeReason {
    SuspectedHack,
    ComplianceViolation,
    UserRequest,      // User sendiri minta freeze (keamanan)
    AdminOverride,
}

/// Log aksi admin
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminLog {
    pub timestamp: i64,
    pub action: String,
    pub target: String,
    pub admin: String,
    pub reason: String,
}

/// Status akun
#[derive(Debug, Clone, PartialEq)]
pub enum AccountStatus {
    Active,
    Frozen(String), // Alasan freeze
}

/// Admin & Compliance Engine
pub struct AdminEngine {
    pub frozen_accounts: HashSet<String>,
    pub admin_addresses: HashSet<String>,
    pub logs: Vec<AdminLog>,
    pub freeze_reasons: HashMap<String, FreezeReason>,
    pub is_emergency_mode: bool,
}

impl AdminEngine {
    pub fn new() -> Self {
        Self {
            frozen_accounts: HashSet::new(),
            admin_addresses: HashSet::new(),
            logs: Vec::new(),
            freeze_reasons: HashMap::new(),
            is_emergency_mode: false,
        }
    }

    /// Daftarkan alamat sebagai admin (hanya di genesis)
    pub fn register_admin(&mut self, address: &str) {
        self.admin_addresses.insert(address.to_string());
    }

    /// Cek apakah alamat adalah admin
    pub fn is_admin(&self, address: &str) -> bool {
        self.admin_addresses.contains(address)
    }

    /// Bekukan akun (hanya admin yang bisa)
    pub fn freeze_account(&mut self, admin: &str, target: &str, reason: FreezeReason) -> Result<String, String> {
        if !self.is_admin(admin) {
            return Err("Unauthorized: only admins can freeze accounts".to_string());
        }

        if self.frozen_accounts.contains(target) {
            return Err(format!("{} is already frozen", target));
        }

        let reason_str = format!("{:?}", reason);
        self.frozen_accounts.insert(target.to_string());
        self.freeze_reasons.insert(target.to_string(), reason);

        self.logs.push(AdminLog {
            timestamp: Utc::now().timestamp(),
            action: "FREEZE".to_string(),
            target: target.to_string(),
            admin: admin.to_string(),
            reason: reason_str.clone(),
        });

        Ok(format!("Account {} frozen: {}", target, reason_str))
    }

    /// Cairkan akun
    pub fn unfreeze_account(&mut self, admin: &str, target: &str) -> Result<String, String> {
        if !self.is_admin(admin) {
            return Err("Unauthorized: only admins can unfreeze accounts".to_string());
        }

        if !self.frozen_accounts.contains(target) {
            return Err(format!("{} is not frozen", target));
        }

        self.frozen_accounts.remove(target);
        self.freeze_reasons.remove(target);

        self.logs.push(AdminLog {
            timestamp: Utc::now().timestamp(),
            action: "UNFREEZE".to_string(),
            target: target.to_string(),
            admin: admin.to_string(),
            reason: "Account restored".to_string(),
        });

        Ok(format!("Account {} unfrozen", target))
    }

    /// Cek status akun (dipanggil sebelum transfer/bid/dll)
    pub fn check_account(&self, address: &str) -> AccountStatus {
        if self.frozen_accounts.contains(address) {
            let reason = self.freeze_reasons.get(address)
                .map(|r| format!("{:?}", r))
                .unwrap_or("Unknown".to_string());
            AccountStatus::Frozen(reason)
        } else {
            AccountStatus::Active
        }
    }

    /// Aktifkan mode darurat (semua transaksi diblokir)
    pub fn activate_emergency(&mut self, admin: &str) -> Result<String, String> {
        if !self.is_admin(admin) {
            return Err("Unauthorized".to_string());
        }

        self.is_emergency_mode = true;
        self.logs.push(AdminLog {
            timestamp: Utc::now().timestamp(),
            action: "EMERGENCY_ON".to_string(),
            target: "NETWORK".to_string(),
            admin: admin.to_string(),
            reason: "Emergency shutdown activated".to_string(),
        });

        Ok("EMERGENCY MODE ACTIVATED - All transactions paused".to_string())
    }

    /// Nonaktifkan mode darurat
    pub fn deactivate_emergency(&mut self, admin: &str) -> Result<String, String> {
        if !self.is_admin(admin) {
            return Err("Unauthorized".to_string());
        }

        self.is_emergency_mode = false;
        self.logs.push(AdminLog {
            timestamp: Utc::now().timestamp(),
            action: "EMERGENCY_OFF".to_string(),
            target: "NETWORK".to_string(),
            admin: admin.to_string(),
            reason: "Emergency mode lifted".to_string(),
        });

        Ok("Emergency mode deactivated - Network resumed".to_string())
    }

    /// Cek apakah transaksi diizinkan (gabungan account + emergency check)
    pub fn can_transact(&self, address: &str) -> Result<(), String> {
        if self.is_emergency_mode {
            return Err("Network in EMERGENCY MODE - all transactions paused".to_string());
        }

        match self.check_account(address) {
            AccountStatus::Active => Ok(()),
            AccountStatus::Frozen(reason) => Err(format!("Account frozen: {}", reason)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freeze_and_unfreeze() {
        let mut admin_engine = AdminEngine::new();
        admin_engine.register_admin("nfm_founder");

        // Freeze akun yang dicurigai di-hack
        assert!(admin_engine.freeze_account("nfm_founder", "nfm_hacked_user", FreezeReason::SuspectedHack).is_ok());
        assert_eq!(admin_engine.check_account("nfm_hacked_user"), AccountStatus::Frozen("SuspectedHack".to_string()));

        // Unfreeze setelah investigasi
        assert!(admin_engine.unfreeze_account("nfm_founder", "nfm_hacked_user").is_ok());
        assert_eq!(admin_engine.check_account("nfm_hacked_user"), AccountStatus::Active);
    }

    #[test]
    fn test_non_admin_cannot_freeze() {
        let mut admin_engine = AdminEngine::new();
        admin_engine.register_admin("nfm_founder");

        let result = admin_engine.freeze_account("nfm_random_user", "nfm_target", FreezeReason::AdminOverride);
        assert!(result.is_err());
    }

    #[test]
    fn test_emergency_mode() {
        let mut admin_engine = AdminEngine::new();
        admin_engine.register_admin("nfm_founder");

        // Aktifkan darurat
        admin_engine.activate_emergency("nfm_founder").unwrap();
        assert!(admin_engine.can_transact("nfm_anyone").is_err());

        // Nonaktifkan darurat
        admin_engine.deactivate_emergency("nfm_founder").unwrap();
        assert!(admin_engine.can_transact("nfm_anyone").is_ok());
    }

    #[test]
    fn test_frozen_account_cannot_transact() {
        let mut admin_engine = AdminEngine::new();
        admin_engine.register_admin("nfm_admin");

        admin_engine.freeze_account("nfm_admin", "nfm_bad_actor", FreezeReason::ComplianceViolation).unwrap();
        assert!(admin_engine.can_transact("nfm_bad_actor").is_err());
        assert!(admin_engine.can_transact("nfm_good_user").is_ok());
    }

    #[test]
    fn test_audit_trail() {
        let mut admin_engine = AdminEngine::new();
        admin_engine.register_admin("nfm_admin");

        admin_engine.freeze_account("nfm_admin", "nfm_user", FreezeReason::UserRequest).unwrap();
        admin_engine.unfreeze_account("nfm_admin", "nfm_user").unwrap();
        admin_engine.activate_emergency("nfm_admin").unwrap();

        assert_eq!(admin_engine.logs.len(), 3);
        assert_eq!(admin_engine.logs[0].action, "FREEZE");
        assert_eq!(admin_engine.logs[1].action, "UNFREEZE");
        assert_eq!(admin_engine.logs[2].action, "EMERGENCY_ON");
    }
}

