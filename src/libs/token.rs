use rand::{distr::Alphanumeric, Rng};

pub fn generate_token(len: usize) -> String {
    let token: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();
    token
}
