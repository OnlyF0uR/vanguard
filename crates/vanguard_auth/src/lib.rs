use std::path::Path;
use tokio::fs;
use fn_dsa::{
    sign_key_size, vrfy_key_size, signature_size,
    FN_DSA_LOGN_512,
    KeyPairGenerator, KeyPairGeneratorStandard,
    SigningKey, SigningKeyStandard,
    VerifyingKey, VerifyingKeyStandard,
    DOMAIN_NONE, HASH_ID_RAW,
};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use cookie::{Cookie, SameSite};
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub struct AuthManager {
    sign_key: Vec<u8>,
    vrfy_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

impl AuthManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let key_path = Path::new(".private.key");
        
        if key_path.exists() {
            let bytes = fs::read(key_path).await?;
            let sign_size = sign_key_size(FN_DSA_LOGN_512);
            let vrfy_size = vrfy_key_size(FN_DSA_LOGN_512);
            
            if bytes.len() == sign_size + vrfy_size {
                let sign_key = bytes[..sign_size].to_vec();
                let vrfy_key = bytes[sign_size..].to_vec();
                return Ok(Self { sign_key, vrfy_key });
            }
        }

        // Generate new keypair
        let mut kg = KeyPairGeneratorStandard::default();
        let mut sign_key = vec![0u8; sign_key_size(FN_DSA_LOGN_512)];
        let mut vrfy_key = vec![0u8; vrfy_key_size(FN_DSA_LOGN_512)];
        kg.keygen(FN_DSA_LOGN_512, &mut OsRng, &mut sign_key, &mut vrfy_key);

        let mut combined = sign_key.clone();
        combined.extend_from_slice(&vrfy_key);
        fs::write(key_path, combined).await?;

        Ok(Self { sign_key, vrfy_key })
    }

    pub fn create_token(&self, subject: &str, duration_mins: i64) -> Result<String, String> {
        let now = Utc::now();
        let claims = Claims {
            sub: subject.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::minutes(duration_mins)).timestamp(),
        };

        let payload = serde_json::to_string(&claims).map_err(|e| e.to_string())?;
        
        let mut sk = SigningKeyStandard::decode(&self.sign_key)
            .ok_or("Failed to decode signing key")?;
        
        let mut sig = vec![0u8; signature_size(sk.get_logn())];
        // sign returns () and modifies sig in place
        sk.sign(&mut OsRng, &DOMAIN_NONE, &HASH_ID_RAW, payload.as_bytes(), &mut sig);
        
        let mut token = STANDARD.encode(payload);
        token.push('.');
        token.push_str(&STANDARD.encode(sig));
        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Option<Claims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 2 { return None; }

        let payload_bytes = STANDARD.decode(parts[0]).ok()?;
        let sig_bytes = STANDARD.decode(parts[1]).ok()?;
        
        let vk = VerifyingKeyStandard::decode(&self.vrfy_key)?;
        
        if vk.verify(&sig_bytes, &DOMAIN_NONE, &HASH_ID_RAW, &payload_bytes) {
            let claims: Claims = serde_json::from_slice(&payload_bytes).ok()?;
            if claims.exp > Utc::now().timestamp() {
                return Some(claims);
            }
        }
        None
    }

    pub fn refresh_token(&self, claims: &Claims, duration_mins: i64) -> Result<String, String> {
        self.create_token(&claims.sub, duration_mins)
    }

    pub fn auth_cookie(&self, token: String, secure: bool) -> String {
        Cookie::build(("vanguard_auth", token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(secure)
            .to_string()
    }
}
