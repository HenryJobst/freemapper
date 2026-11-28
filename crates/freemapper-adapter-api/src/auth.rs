// SPDX-License-Identifier: AGPL-3.0-or-later

//! HTTP Basic credentials for a region: which host takes them, which environment variables hold
//! them and what a person orders to get an account that works (the manifest's `[auth]`).

use crate::error::AdapterError;

/// Where a region takes credentials and what a person needs for them. The manifest's `[auth]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSpec {
    /// The region in English, for the message that asks for the credentials.
    pub name: String,
    /// The one host that gets the `Authorization` header.
    pub host: String,
    /// The environment variable of the user name.
    pub user_env: String,
    /// The environment variable of the password.
    pub password_env: String,
    /// What a person orders to get an account that works.
    pub products: Vec<String>,
}

/// A password. It has no `Debug` and no `Display`, so it cannot reach a message or a log by
/// accident; [`Password::expose`] is the one way to read it.
///
/// ```compile_fail
/// let password = freemapper_adapter_api::auth::Password::new("x".to_string());
/// println!("{password:?}");
/// ```
pub struct Password(String);

impl Password {
    pub fn new(text: String) -> Self {
        Self(text)
    }

    /// The text, for the one place that builds the `Authorization` header.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

/// A user name and its password.
pub struct Credentials {
    pub user: String,
    pub password: Password,
}

/// The credentials `spec` names, read through `env` (the process environment in the program,
/// a table in the tests). A variable that is missing or blank is the error that names both
/// variables and the products to order; the password is in no message.
pub fn credentials(
    spec: &AuthSpec,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<Credentials, AdapterError> {
    let read = |name: &str| env(name).filter(|value| !value.trim().is_empty());
    match (read(&spec.user_env), read(&spec.password_env)) {
        (Some(user), Some(password)) => Ok(Credentials {
            user,
            password: Password::new(password),
        }),
        _ => Err(AdapterError::CredentialsMissing {
            name: spec.name.clone(),
            user_env: spec.user_env.clone(),
            password_env: spec.password_env.clone(),
            products: spec.products.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> AuthSpec {
        AuthSpec {
            name: "Sweden".to_string(),
            host: "dl.example.org".to_string(),
            user_env: "FREEMAPPER_X_USER".to_string(),
            password_env: "FREEMAPPER_X_PASSWORD".to_string(),
            products: vec!["Product A".to_string(), "Product B".to_string()],
        }
    }

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_string())
        }
    }

    #[test]
    fn credentials_are_read_from_the_two_variables() {
        let found = credentials(
            &spec(),
            &env(&[
                ("FREEMAPPER_X_USER", "person@example.org"),
                ("FREEMAPPER_X_PASSWORD", "s3cret-πß"),
            ]),
        )
        .unwrap();
        assert_eq!(found.user, "person@example.org");
        assert_eq!(found.password.expose(), "s3cret-πß");
    }

    #[test]
    fn a_missing_or_empty_variable_names_both_variables_and_the_products() {
        for pairs in [
            vec![],
            vec![("FREEMAPPER_X_USER", "u")],
            vec![("FREEMAPPER_X_USER", "u"), ("FREEMAPPER_X_PASSWORD", "")],
            vec![("FREEMAPPER_X_USER", "  "), ("FREEMAPPER_X_PASSWORD", "p")],
        ] {
            let Err(error) = credentials(&spec(), &env(&pairs)) else {
                panic!("{pairs:?} gave credentials");
            };
            let text = error.to_string();
            for part in [
                "Sweden",
                "FREEMAPPER_X_USER",
                "FREEMAPPER_X_PASSWORD",
                "Product A",
                "Product B",
            ] {
                assert!(text.contains(part), "{part} missing in: {text}");
            }
        }
    }

    #[test]
    fn the_password_is_in_no_message_and_no_debug_print() {
        let secret = "s3cret-πß";
        let spec = spec();
        let found = credentials(
            &spec,
            &env(&[
                ("FREEMAPPER_X_USER", "u"),
                ("FREEMAPPER_X_PASSWORD", secret),
            ]),
        )
        .unwrap();
        assert_eq!(found.password.expose(), secret);
        assert!(!format!("{spec:?}").contains(secret));
        let Err(error) = credentials(&spec, &env(&[("FREEMAPPER_X_PASSWORD", secret)])) else {
            panic!("a user name is missing");
        };
        assert!(!error.to_string().contains(secret));
        assert!(!format!("{error:?}").contains(secret));
    }
}
