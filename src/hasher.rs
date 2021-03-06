use crate::config::{HashAlgorithm, PasswordSettings};

pub fn hash_password(password: &str, settings: &PasswordSettings) -> Option<String> {
    return match settings.algorithm() {
        #[cfg(feature = "argon2")]
        HashAlgorithm::Argon2 => hash_argon2(password),
        #[cfg(feature = "scrypt")]
        HashAlgorithm::Scrypt => hash_scrypt(password),
        #[cfg(feature = "pbkdf2")]
        HashAlgorithm::PBKDF2SHA256 => hash_pbkdf2_sha256(password, settings.pbkdf2_iterations()),
        #[cfg(feature = "pbkdf2")]
        HashAlgorithm::PBKDF2SHA512 => hash_pbkdf2_sha512(password, settings.pbkdf2_iterations()),
        HashAlgorithm::None => Some(password.to_string()),
    };
}

pub fn check_password(
    password: &str,
    settings: &PasswordSettings,
    comparison: &str,
) -> Option<bool> {
    return match settings.algorithm() {
        #[cfg(feature = "argon2")]
        HashAlgorithm::Argon2 => compare_argon2(password, comparison),
        #[cfg(feature = "scrypt")]
        HashAlgorithm::Scrypt => compare_scrypt(password, comparison),
        #[cfg(feature = "pbkdf2")]
        HashAlgorithm::PBKDF2SHA256 | HashAlgorithm::PBKDF2SHA512 => {
            compare_pbkdf2(password, comparison)
        }
        HashAlgorithm::None => Some(password == comparison),
    };
}

#[cfg(feature = "argon2")]
fn hash_argon2(password: &str) -> Option<String> {
    use argon2::password_hash::{PasswordHasher, SaltString};

    let mut rng = rand::thread_rng();
    let salt_string = SaltString::generate(&mut rng);
    let hasher = argon2::Argon2::default();

    return Some(
        hasher
            .hash_password_simple(password.as_bytes(), salt_string.as_ref())
            .ok()?
            .to_string(),
    );
}

#[cfg(feature = "scrypt")]
fn hash_scrypt(password: &str) -> Option<String> {
    use scrypt::password_hash::{PasswordHasher, SaltString};

    let mut rng = rand::thread_rng();
    let salt_string = SaltString::generate(&mut rng);

    return Some(
        scrypt::Scrypt
            .hash_password_simple(password.as_bytes(), salt_string.as_ref())
            .ok()?
            .to_string(),
    );
}

#[cfg(feature = "pbkdf2")]
fn hash_pbkdf2_sha256(password: &str, iterations: usize) -> Option<String> {
    use pbkdf2::password_hash::{Ident, PasswordHasher, SaltString};

    let mut rng = rand::thread_rng();
    let salt_string = SaltString::generate(&mut rng);

    return pbkdf2::Pbkdf2
        .hash_password(
            password.as_bytes(),
            Some(Ident::new(&pbkdf2::Algorithm::Pbkdf2Sha256.to_string())),
            None,
            pbkdf2::Params {
                rounds: iterations as u32,
                output_length: 32,
            },
            salt_string.as_salt(),
        )
        .ok()
        .map(|r| r.to_string());
}

#[cfg(feature = "pbkdf2")]
fn hash_pbkdf2_sha512(password: &str, iterations: usize) -> Option<String> {
    use pbkdf2::password_hash::{Ident, PasswordHasher, SaltString};

    let mut rng = rand::thread_rng();
    let salt_string = SaltString::generate(&mut rng);

    return pbkdf2::Pbkdf2
        .hash_password(
            password.as_bytes(),
            Some(Ident::new(&pbkdf2::Algorithm::Pbkdf2Sha512.to_string())),
            None,
            pbkdf2::Params {
                rounds: iterations as u32,
                output_length: 64,
            },
            salt_string.as_salt(),
        )
        .ok()
        .map(|r| r.to_string());
}

#[cfg(feature = "argon2")]
fn compare_argon2(password: &str, comp: &str) -> Option<bool> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;

    let parsed_hash = PasswordHash::new(comp).ok()?;
    return Some(
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
    );
}

#[cfg(feature = "scrypt")]
fn compare_scrypt(password: &str, comp: &str) -> Option<bool> {
    use scrypt::password_hash::{PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(comp).ok()?;
    return Some(
        scrypt::Scrypt
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
    );
}

#[cfg(feature = "pbkdf2")]
fn compare_pbkdf2(password: &str, comp: &str) -> Option<bool> {
    use pbkdf2::password_hash::{PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(comp).ok()?;
    return Some(
        pbkdf2::Pbkdf2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "argon2")]
    mod argon2 {
        use super::*;

        #[test]
        fn test_argon2() {
            assert!(hash_argon2("password").unwrap().len() > 0);
        }

        #[test]
        fn test_argon2_check_1() {
            let comp = hash_argon2("password").unwrap();
            assert!(compare_argon2("password", &comp).unwrap());
        }

        #[test]
        fn test_argon2_check_2() {
            let comp = hash_argon2("password").unwrap();
            assert!(!compare_argon2("password2", &comp).unwrap());
        }
    }

    #[cfg(feature = "scrypt")]
    mod scrypt {
        use super::*;

        #[test]
        fn test_scrypt() {
            assert!(hash_scrypt("password").unwrap().len() > 0);
        }

        #[test]
        fn test_scrypt_check_1() {
            let comp = hash_scrypt("password").unwrap();
            assert!(compare_scrypt("password", &comp).unwrap());
        }

        #[test]
        fn test_scrypt_check_2() {
            let comp = hash_scrypt("password").unwrap();
            assert!(!compare_scrypt("password2", &comp).unwrap());
        }
    }

    #[cfg(feature = "pbkdf2")]
    mod pbkdf2 {
        use super::*;

        #[test]
        fn test_pbkdf2_sha256() {
            assert!(hash_pbkdf2_sha256("password", 10_000).unwrap().len() > 0);
        }

        #[test]
        fn test_pbkdf2_sha512() {
            assert!(hash_pbkdf2_sha512("password", 10_000).unwrap().len() > 0);
        }

        #[test]
        fn test_pbkdf2_sha256_check_1() {
            let comp = hash_pbkdf2_sha256("password", 10_000).unwrap();
            assert!(compare_pbkdf2("password", &comp).unwrap());
        }

        #[test]
        fn test_pbkdf2_sha256_check_2() {
            let comp = hash_pbkdf2_sha256("password", 10_000).unwrap();
            assert!(!compare_pbkdf2("password2", &comp).unwrap());
        }

        #[test]
        fn test_pbkdf2_sha512_check_1() {
            let comp = hash_pbkdf2_sha512("password", 10_000).unwrap();
            assert!(compare_pbkdf2("password", &comp).unwrap());
        }

        #[test]
        fn test_pbkdf2_sha512_check_2() {
            let comp = hash_pbkdf2_sha512("password", 10_000).unwrap();
            assert!(!compare_pbkdf2("password2", &comp).unwrap());
        }
    }
}
