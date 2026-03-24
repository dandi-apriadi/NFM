#![allow(dead_code)]
use serde::{Serialize, Deserialize};

/// Tipe diskon yang tersedia di ekosistem NFM
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CouponType {
    Founder,       // 60% discount - Soulbound, hanya 1
    ApexWorkhorse, // 50% discount - Top 5 kontributor tahunan
    GenesisTier1,  // 30% discount - 100 pertama (Permanen)
    GenesisTier2,  // 20% discount - 500 pertama (Permanen)
    GenesisTier3,  // 10% discount - 2000 pertama (Permanen)
    GenesisTier4,  // 5% discount - 5000 pertama (Permanen)
    GenesisTier5,  // 2.5% discount - 10000 pertama (Permanen)
    GenesisTier6,  // 2.5% discount - 20000 pertama (7 hari, stackable dengan T1-T5)
}

/// Kupon diskon milik seorang pengguna
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Coupon {
    pub coupon_type: CouponType,
    pub discount_percent: f64,
    pub is_permanent: bool,
    pub owner: String, // NFM address pemilik
}

impl Coupon {
    pub fn new(coupon_type: CouponType, owner: &str) -> Self {
        let (discount, permanent) = match &coupon_type {
            CouponType::Founder       => (60.0, true),
            CouponType::ApexWorkhorse  => (50.0, true),
            CouponType::GenesisTier1   => (30.0, true),
            CouponType::GenesisTier2   => (20.0, true),
            CouponType::GenesisTier3   => (10.0, true),
            CouponType::GenesisTier4   => (5.0, true),
            CouponType::GenesisTier5   => (2.5, true),
            CouponType::GenesisTier6   => (2.5, false), // Sementara (7 hari)
        };

        Self {
            coupon_type,
            discount_percent: discount,
            is_permanent: permanent,
            owner: owner.to_string(),
        }
    }
}

/// Registry kuota Genesis Coupon
#[derive(Debug)]
pub struct CouponRegistry {
    pub coupons: Vec<Coupon>,
    pub tier1_claimed: u32,  // max 100
    pub tier2_claimed: u32,  // max 500
    pub tier3_claimed: u32,  // max 2000
    pub tier4_claimed: u32,  // max 5000
    pub tier5_claimed: u32,  // max 10000
    pub tier6_claimed: u32,  // max 20000
}

impl CouponRegistry {
    pub fn new() -> Self {
        Self {
            coupons: Vec::new(),
            tier1_claimed: 0,
            tier2_claimed: 0,
            tier3_claimed: 0,
            tier4_claimed: 0,
            tier5_claimed: 0,
            tier6_claimed: 0,
        }
    }

    /// Klaim Genesis Coupon (cek kuota). Mengembalikan None jika kuota habis.
    pub fn claim_genesis(&mut self, tier: CouponType, owner: &str) -> Option<Coupon> {
        let (counter, max) = match &tier {
            CouponType::GenesisTier1 => (&mut self.tier1_claimed, 100),
            CouponType::GenesisTier2 => (&mut self.tier2_claimed, 500),
            CouponType::GenesisTier3 => (&mut self.tier3_claimed, 2000),
            CouponType::GenesisTier4 => (&mut self.tier4_claimed, 5000),
            CouponType::GenesisTier5 => (&mut self.tier5_claimed, 10000),
            CouponType::GenesisTier6 => (&mut self.tier6_claimed, 20000),
            _ => return None, // Founder/Apex bukan Genesis
        };

        if *counter >= max {
            return None; // Kuota habis
        }

        *counter += 1;
        let coupon = Coupon::new(tier, owner);
        self.coupons.push(coupon.clone());
        Some(coupon)
    }

    /// Hitung total diskon aktif untuk seorang user (stacking T6 + salah satu T1-T5)
    pub fn get_discount_for(&self, owner: &str) -> f64 {
        let user_coupons: Vec<&Coupon> = self.coupons.iter()
            .filter(|c| c.owner == owner)
            .collect();

        if user_coupons.is_empty() {
            return 0.0;
        }

        // Cek Founder/Apex (override tertinggi, tidak di-stack)
        for c in &user_coupons {
            if c.coupon_type == CouponType::Founder || c.coupon_type == CouponType::ApexWorkhorse {
                return c.discount_percent;
            }
        }

        // Ambil diskon terbesar dari T1-T5
        let main_discount = user_coupons.iter()
            .filter(|c| c.coupon_type != CouponType::GenesisTier6)
            .map(|c| c.discount_percent)
            .fold(0.0_f64, f64::max);

        // Cek T6 bonus (stackable)
        let has_tier6 = user_coupons.iter()
            .any(|c| c.coupon_type == CouponType::GenesisTier6);

        if has_tier6 {
            main_discount + 2.5 // Stack T6 di atas T1-T5
        } else {
            main_discount
        }
    }

    /// Hitung biaya AI setelah diskon
    pub fn apply_discount(&self, owner: &str, base_fee: f64) -> f64 {
        let discount = self.get_discount_for(owner);
        let discounted = base_fee * (1.0 - discount / 100.0);
        (discounted * 10000.0).round() / 10000.0 // 4 desimal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_founder_discount_60_percent() {
        let mut reg = CouponRegistry::new();
        let coupon = Coupon::new(CouponType::Founder, "nfm_founder");
        reg.coupons.push(coupon);

        let fee = reg.apply_discount("nfm_founder", 10.0);
        assert_eq!(fee, 4.0); // 10 - 60% = 4
    }

    #[test]
    fn test_genesis_tier1_discount() {
        let mut reg = CouponRegistry::new();
        reg.claim_genesis(CouponType::GenesisTier1, "nfm_early_user");

        let fee = reg.apply_discount("nfm_early_user", 10.0);
        assert_eq!(fee, 7.0); // 10 - 30% = 7
    }

    #[test]
    fn test_tier6_stacking() {
        let mut reg = CouponRegistry::new();
        reg.claim_genesis(CouponType::GenesisTier3, "nfm_user_x"); // 10%
        reg.claim_genesis(CouponType::GenesisTier6, "nfm_user_x"); // +2.5%

        let discount = reg.get_discount_for("nfm_user_x");
        assert_eq!(discount, 12.5); // 10% + 2.5%

        let fee = reg.apply_discount("nfm_user_x", 100.0);
        assert_eq!(fee, 87.5); // 100 - 12.5% = 87.5
    }

    #[test]
    fn test_quota_limit() {
        let mut reg = CouponRegistry::new();

        // Klaim 100 T1 (max kuota)
        for i in 0..100 {
            let result = reg.claim_genesis(CouponType::GenesisTier1, &format!("nfm_user_{}", i));
            assert!(result.is_some());
        }

        // Klaim ke-101 harus gagal
        let overflow = reg.claim_genesis(CouponType::GenesisTier1, "nfm_user_overflow");
        assert!(overflow.is_none(), "Should reject when quota is full");
    }
}

