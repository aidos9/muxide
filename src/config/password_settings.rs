use serde::{Deserialize, Serialize};

#[inline]
const fn serde_default_as_false() -> bool {
    return false;
}

fn default_password_file_location() -> String {
    if let Some(mut path) = dirs::home_dir() {
        path.push(".config/muxide/password");

        return path.to_str().map(|s| s.to_string()).unwrap();
    } else {
        return String::from("~/.config/muxide/password");
    }
}

#[cfg(feature = "pbkdf2")]
fn default_pbkdf2_iterations() -> usize {
    return pbkdf2::Params::default().rounds as usize;
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PasswordSettings {
    #[serde(default)]
    hash_algorithm: HashAlgorithm,
    #[cfg(feature = "pbkdf2")]
    #[serde(default = "default_pbkdf2_iterations")]
    pbkdf2_iterations: usize,
    #[serde(default = "default_password_file_location")]
    password_file_location: String,
    #[serde(default = "serde_default_as_false")]
    disable_prompt_for_new_password: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    #[cfg(feature = "argon2")]
    Argon2,
    #[cfg(feature = "scrypt")]
    Scrypt,
    #[cfg(feature = "pbkdf2")]
    PBKDF2SHA256,
    #[cfg(feature = "pbkdf2")]
    PBKDF2SHA512,
    None,
}

impl PasswordSettings {
    pub fn algorithm(&self) -> HashAlgorithm {
        return self.hash_algorithm;
    }

    #[cfg(feature = "pbkdf2")]
    pub fn pbkdf2_iterations(&self) -> usize {
        return self.pbkdf2_iterations;
    }

    pub fn password_file_location(&self) -> &String {
        return &self.password_file_location;
    }

    pub fn disable_prompt_for_new_password(&self) -> bool {
        return self.disable_prompt_for_new_password;
    }
}

impl Default for PasswordSettings {
    fn default() -> Self {
        return Self {
            hash_algorithm: HashAlgorithm::default(),
            password_file_location: default_password_file_location(),
            #[cfg(feature = "pbkdf2")]
            pbkdf2_iterations: default_pbkdf2_iterations(),
            disable_prompt_for_new_password: false,
        };
    }
}

#[cfg(feature = "argon2")]
impl Default for HashAlgorithm {
    fn default() -> Self {
        return Self::Argon2;
    }
}

#[cfg(not(feature = "argon2"))]
impl Default for HashAlgorithm {
    fn default() -> Self {
        return Self::None;
    }
}

impl HashAlgorithm {
    pub fn supported_algorithms() -> String {
        let mut algorithms = "[".to_string();

        #[cfg(feature = "argon2")]
        algorithms.push_str("Argon2, ");

        #[cfg(feature = "pbkdf2")]
        algorithms.push_str("PBKDF2_SHA256, PBKDF2_SHA512, ");
        #[cfg(feature = "scrypt")]
        algorithms.push_str("Scrypt, ");

        if algorithms.chars().last() == Some(',') {
            algorithms.pop();
        }

        algorithms.push(']');

        return algorithms;
    }
}

impl<'de> Deserialize<'de> for HashAlgorithm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut string: String = Deserialize::deserialize(deserializer)?;
        string = string.to_lowercase();

        return Ok(match string.as_str() {
            #[cfg(feature = "argon2")]
            "argon2" => Self::Argon2,
            #[cfg(feature = "scrypt")]
            "scrypt" => Self::Scrypt,
            #[cfg(feature = "pbkdf2")]
            "pbkdf2_sha256" => Self::PBKDF2SHA256,
            #[cfg(feature = "pbkdf2")]
            "pbkdf2_sha512" => Self::PBKDF2SHA512,
            "none" => Self::None,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Expected a supported hash algorithm. Supported algorithms = {}",
                    HashAlgorithm::supported_algorithms()
                )))
            }
        });
    }
}

impl Serialize for HashAlgorithm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string = match self {
            #[cfg(feature = "argon2")]
            HashAlgorithm::Argon2 => "Argon2",
            #[cfg(feature = "scrypt")]
            HashAlgorithm::Scrypt => "Scrypt",
            #[cfg(feature = "pbkdf2")]
            HashAlgorithm::PBKDF2SHA256 => "PBKDF2_SHA256",
            #[cfg(feature = "pbkdf2")]
            HashAlgorithm::PBKDF2SHA512 => "PBKDF2_SHA512",
            HashAlgorithm::None => "None",
        };

        return Serialize::serialize(string, serializer);
    }
}
