use argon2::{
    Argon2,
    password_hash::{
        PasswordHasher, SaltString, rand_core::OsRng, PasswordHash, PasswordVerifier
    },
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng); // OsRng는 Zero-Cost Type이므로 메모리 할당 없이 사용 가능
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}

pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
    let parse_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parse_hash).is_ok())
}