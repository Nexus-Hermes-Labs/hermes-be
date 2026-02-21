use argon2::{
    password_hash::{SaltString, PasswordHasher},
    Argon2,
};

fn main() {
    let argon2 = Argon2::default();
    let password = "Password123!";
    let salt = SaltString::generate(rand::thread_rng());
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();
    println!("{}", hash);
}
