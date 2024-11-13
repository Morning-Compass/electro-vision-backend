use jsonwebtoken::{decode, errors::Error, Algorithm, DecodingKey, TokenData, Validation};

use crate::auth::jwt::Claims;

pub fn jwt_decode(token: String) -> Result<TokenData<Claims>, Error> {
    let secret_key = dotenv::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["iat", "exp"]);
    decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &validation,
    )
}
