use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterUser {
    pub email: String,
    pub password: String,
    pub nama_lengkap: Option<String>,
    pub no_hp: Option<String>,
    pub foto_profile: Option<String>,
    pub bio: Option<String>,
}
