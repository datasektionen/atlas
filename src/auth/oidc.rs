use std::borrow::Cow;

use log::*;
use openidconnect::{
    AdditionalClaims, AsyncHttpClient, AuthenticationFlow, AuthorizationCode, Client, ClientId,
    ClientSecret, CsrfToken, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IdTokenFields, IssuerUrl, Nonce, RedirectUrl, Scope, StandardErrorResponse,
    StandardTokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
        CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreProviderMetadata,
        CoreResponseType, CoreRevocableToken, CoreRevocationErrorResponse,
        CoreTokenIntrospectionResponse, CoreTokenType,
    },
};
use rocket::http::uri::Origin;
use serde::{Deserialize, Serialize};

use super::{Session, hive::HivePermission};

pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(thiserror::Error, Debug)]
pub enum OidcInitError<E: 'static + std::error::Error> {
    #[error("invalid OIDC issuer URL: {0}")]
    InvalidIssuerUrl(#[from] openidconnect::url::ParseError),
    #[error("OIDC provider metadata discovery failure: {0}")]
    DiscoveryFailure(#[from] openidconnect::DiscoveryError<E>),
}

#[derive(thiserror::Error, Debug)]
pub enum OidcAuthenticationError {
    #[error("invalid OIDC issuer URL: {0}")]
    InvalidRedirectUrl(#[from] openidconnect::url::ParseError),
    #[error("OIDC issuer does not publish a user info endpoint")]
    NoUserInfoUrl,
    #[error("OIDC issuer returned a CSRF token `{0}` that does not match ours")]
    BadCsrfToken(String),
    #[error("failed to exchange code with token from OIDC issuer")]
    CodeTokenExchangeFailure,
    #[error("OIDC issuer did not return an ID token")]
    NoIdToken,
    #[error("OIDC issuer returned an ID token, but it failed verification")]
    BadIdToken(#[from] openidconnect::ClaimsVerificationError),
    #[error("OIDC issuer did not return any `name` claim for the subject")]
    NoNameClaim,
}

type ReqwestError<'c> = <openidconnect::reqwest::Client as AsyncHttpClient<'c>>::Error;

// must be persisted between start of login flow and OIDC callback (end of flow)
#[derive(Serialize, Deserialize)]
pub struct OidcAuthenticationContext<'n> {
    redirect_url: RedirectUrl,
    next: Option<Origin<'n>>,
    csrf_state: CsrfToken,
    nonce: Nonce,
}

pub struct OidcAuthenticationResult<'n> {
    pub session: Session,
    pub next: Option<Origin<'n>>,
}

pub type OidcTokenResponse = StandardTokenResponse<OidcIdTokenFields, CoreTokenType>;

pub type OidcIdTokenFields = IdTokenFields<
    OidcAdditionalClaims,
    EmptyExtraTokenFields,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct OidcAdditionalClaims {
    permissions: Vec<HivePermission>,
}

impl AdditionalClaims for OidcAdditionalClaims {}

// wrapper for simplicity and impl'ing
pub struct OidcClient {
    client: Client<
        // Custom additional claim 'permissions'
        OidcAdditionalClaims,
        CoreAuthDisplay,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJsonWebKey,
        CoreAuthPrompt,
        StandardErrorResponse<CoreErrorResponseType>,
        // TokenResponse also references additional claims
        OidcTokenResponse,
        CoreTokenIntrospectionResponse,
        CoreRevocableToken,
        CoreRevocationErrorResponse,
        // Has auth URL
        EndpointSet,
        // Has device auth URL
        EndpointNotSet,
        // Has introspection URL
        EndpointNotSet,
        // Has revocation URL
        EndpointNotSet,
        // Has token URL
        EndpointMaybeSet,
        // Has user info URL
        EndpointMaybeSet,
    >,
    http_client: openidconnect::reqwest::Client,
}

impl OidcClient {
    pub async fn new<'c>(config: OidcConfig) -> Result<Self, OidcInitError<ReqwestError<'c>>> {
        info!("new");
        let issuer_url = IssuerUrl::new(config.issuer_url)?;

        let http_client = openidconnect::reqwest::ClientBuilder::new()
            .redirect(openidconnect::reqwest::redirect::Policy::none()) // prevent SSRF attacks
            .build()
            .expect("reqwest client"); // there should be no reason for it to fail

        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer_url, &http_client).await?;

        let client_id = ClientId::new(config.client_id);
        let client_secret = ClientSecret::new(config.client_secret);

        let client =
            Client::from_provider_metadata(provider_metadata, client_id, Some(client_secret));

        Ok(Self {
            client,
            http_client,
        })
    }

    pub(super) async fn begin_authentication<'n>(
        &self,
        redirect_url: String,
        next: Option<Origin<'n>>,
    ) -> Result<(String, OidcAuthenticationContext<'n>), OidcAuthenticationError> {
        // SECURITY: Atlas trusts the `Host` header even though it's client-controlled, since it
        // assumes that it's always served behind a reverse proxy and we only received the request
        // in the first place because the `Host` was correctly set

        let redirect_url = RedirectUrl::new(redirect_url)?;

        let (authorize_url, csrf_state, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("profile".to_owned()))
            .add_scope(Scope::new("permissions".to_owned()))
            .set_redirect_uri(Cow::Borrowed(&redirect_url))
            .url();

        let context = OidcAuthenticationContext {
            redirect_url,
            next,
            csrf_state,
            nonce,
        };

        Ok((authorize_url.to_string(), context))
    }

    pub(super) async fn finish_authentication<'n>(
        &self,
        context: OidcAuthenticationContext<'n>,
        code: &str,
        state: &str,
    ) -> Result<OidcAuthenticationResult<'n>, OidcAuthenticationError> {
        if CsrfToken::new(state.to_owned()) != context.csrf_state {
            return Err(OidcAuthenticationError::BadCsrfToken(state.to_owned()));
        }

        // trade code for a token
        let response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .map_err(|_| OidcAuthenticationError::NoUserInfoUrl)?
            .set_redirect_uri(Cow::Borrowed(&context.redirect_url))
            .request_async(&self.http_client)
            .await
            .inspect_err(|e| error!("OIDC code exchange error: {e:?}"))
            .map_err(|_| OidcAuthenticationError::CodeTokenExchangeFailure)?;

        let id_token_verifier = self.client.id_token_verifier();
        let claims = response
            .extra_fields()
            .id_token()
            .ok_or(OidcAuthenticationError::NoIdToken)?
            .claims(&id_token_verifier, &context.nonce)?;

        let end_user_name = claims
            .name()
            .and_then(|claim| claim.iter().next())
            .map(|value| value.1)
            .ok_or(OidcAuthenticationError::NoNameClaim)?;

        let additional_claims = claims.additional_claims();

        let session = Session {
            username: claims.subject().to_string(),
            display_name: end_user_name.to_string(),
            permissions: additional_claims.permissions.clone().into(),
            expiration: claims.expiration().into(),
        };

        Ok(OidcAuthenticationResult {
            session,
            next: context.next,
        })
    }
}
