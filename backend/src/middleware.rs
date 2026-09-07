/// Rate limiting, geo-restriction and security-header middleware for InheritX.
use std::{
    collections::HashSet,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{HeaderValue, Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use dashmap::DashMap;

/// Configuration knobs for the rate limiter.
/// Defaults: 100 requests per 60-second window.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u64,
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }
}

#[derive(Debug)]
struct RateLimitState {
    count: u64,
    window_start: Instant,
}

/// Thread-safe store of per-IP rate-limit state.
#[derive(Clone, Default)]
pub struct RateLimitStore(Arc<DashMap<IpAddr, RateLimitState>>);

impl RateLimitStore {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }

    /// Returns true when the request is within the allowed rate.
    /// Returns false when the caller should respond with 429.
    pub fn check_and_increment(&self, ip: IpAddr, cfg: &RateLimitConfig) -> bool {
        let now = Instant::now();
        let mut entry = self.0.entry(ip).or_insert_with(|| RateLimitState {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start) >= cfg.window {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;
        entry.count <= cfg.max_requests
    }
}

/// Axum middleware function for rate limiting.
pub async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next,
    store: RateLimitStore,
    config: Arc<RateLimitConfig>,
) -> Response<Body> {
    let ip = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));

    if !store.check_and_increment(ip, &config) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too Many Requests - rate limit exceeded. Please slow down.",
        )
            .into_response();
    }

    next.run(req).await
}

/// ISO 3166-1 alpha-2 codes for jurisdictions subject to OFAC comprehensive
/// sanctions programs (Cuba, Iran, North Korea, Syria). This is the default
/// deny-list; override or extend it by setting `OFAC_BLOCKED_COUNTRIES` to a
/// comma-separated list of ISO codes.
pub const DEFAULT_SANCTIONED_COUNTRIES: &[&str] = &["CU", "IR", "KP", "SY"];

/// Configuration for the sanctions-region guard.
#[derive(Debug, Clone)]
pub struct GeoGuardConfig {
    blocked_countries: HashSet<String>,
}

impl GeoGuardConfig {
    pub fn new(blocked_countries: impl IntoIterator<Item = String>) -> Self {
        Self {
            blocked_countries: blocked_countries
                .into_iter()
                .map(|c| c.trim().to_ascii_uppercase())
                .filter(|c| !c.is_empty())
                .collect(),
        }
    }

    /// Reads the deny-list from `OFAC_BLOCKED_COUNTRIES`, falling back to
    /// [`DEFAULT_SANCTIONED_COUNTRIES`] when the variable is unset or empty.
    pub fn from_env() -> Self {
        Self::from_override(std::env::var("OFAC_BLOCKED_COUNTRIES").ok())
    }

    /// Builds the config from an already-read `OFAC_BLOCKED_COUNTRIES` value
    /// (comma-separated ISO codes), falling back to
    /// [`DEFAULT_SANCTIONED_COUNTRIES`] when `None` or blank. Split out from
    /// [`Self::from_env`] so the parsing logic is testable without mutating
    /// process-wide environment state.
    pub fn from_override(env_value: Option<String>) -> Self {
        match env_value {
            Some(value) if !value.trim().is_empty() => {
                Self::new(value.split(',').map(|c| c.to_string()))
            }
            _ => Self::new(DEFAULT_SANCTIONED_COUNTRIES.iter().map(|c| c.to_string())),
        }
    }

    pub fn is_blocked(&self, country: &str) -> bool {
        self.blocked_countries
            .contains(country.trim().to_ascii_uppercase().as_str())
    }
}

impl Default for GeoGuardConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Pluggable IP → ISO country-code resolver, e.g. a local MaxMind DB. Lets the
/// geo-restriction middleware fall back to a real lookup when the request
/// didn't arrive with a `CF-IPCountry` edge header already attached.
pub trait CountryResolver: Send + Sync {
    fn lookup_country(&self, ip: IpAddr) -> Option<String>;
}

/// MaxMind GeoIP2/GeoLite2 country database, loaded from a local `.mmdb`
/// file. Only available with the `geoip` feature enabled.
#[cfg(feature = "geoip")]
pub struct MaxMindGeoIp {
    reader: maxminddb::Reader<Vec<u8>>,
}

#[cfg(feature = "geoip")]
impl MaxMindGeoIp {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            reader: maxminddb::Reader::open_readfile(path)?,
        })
    }
}

#[cfg(feature = "geoip")]
impl CountryResolver for MaxMindGeoIp {
    fn lookup_country(&self, ip: IpAddr) -> Option<String> {
        let result = self.reader.lookup(ip).ok()?;
        result
            .decode_path::<String>(&maxminddb::path!["country", "iso_code"])
            .ok()
            .flatten()
    }
}

/// Builds a [`CountryResolver`] from `GEOIP_DB_PATH` when the `geoip` feature
/// is compiled in and the database loads successfully. Returns `None`
/// (falling back to the `CF-IPCountry` header only) otherwise.
pub fn geoip_resolver_from_env() -> Option<Arc<dyn CountryResolver>> {
    #[cfg(feature = "geoip")]
    {
        let path = std::env::var("GEOIP_DB_PATH").ok()?;
        match MaxMindGeoIp::open(&path) {
            Ok(resolver) => Some(Arc::new(resolver) as Arc<dyn CountryResolver>),
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    path = %path,
                    "failed to load MaxMind GeoIP database; falling back to CF-IPCountry header only"
                );
                None
            }
        }
    }
    #[cfg(not(feature = "geoip"))]
    {
        None
    }
}

/// Cloudflare sends "XX" for IPs it can't place (bogon ranges, some
/// anonymizers) and "T1" for Tor exit nodes — neither is a real country, so
/// treat them as unresolved rather than as a block/allow signal.
fn is_unresolved_country_code(code: &str) -> bool {
    code.is_empty() || code.eq_ignore_ascii_case("XX") || code.eq_ignore_ascii_case("T1")
}

/// Resolves the caller's country: prefers the `CF-IPCountry` edge header
/// (present when the app sits behind Cloudflare), falling back to a
/// `CountryResolver` (e.g. MaxMind DB) lookup by connecting IP.
fn resolve_country(
    req: &Request<Body>,
    ip: IpAddr,
    geoip: Option<&Arc<dyn CountryResolver>>,
) -> Option<String> {
    if let Some(header_country) = req
        .headers()
        .get("CF-IPCountry")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|c| !is_unresolved_country_code(c))
    {
        return Some(header_country.to_string());
    }

    geoip.and_then(|resolver| resolver.lookup_country(ip))
}

/// Axum middleware that blocks requests originating from OFAC-sanctioned
/// regions, per [`GeoGuardConfig`]. The caller's country is resolved from the
/// `CF-IPCountry` header first, then an optional MaxMind DB lookup. Requests
/// whose country can't be determined are allowed through.
pub async fn geo_restriction_middleware(
    req: Request<Body>,
    next: Next,
    config: Arc<GeoGuardConfig>,
    geoip: Option<Arc<dyn CountryResolver>>,
) -> Response<Body> {
    let ip = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));

    if let Some(country) = resolve_country(&req, ip, geoip.as_ref()) {
        if config.is_blocked(&country) {
            tracing::warn!(country = %country, ip = %ip, "blocked request from sanctioned region");
            return (
                StatusCode::FORBIDDEN,
                "Access denied: requests from this region are restricted.",
            )
                .into_response();
        }
    }

    next.run(req).await
}

/// HSTS layer: max-age=1 year, includeSubDomains, preload.
pub fn hsts_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue> {
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    )
}

/// Content-Security-Policy layer.
pub fn csp_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue> {
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
    )
}

/// X-Frame-Options: DENY layer.
pub fn x_frame_options_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue> {
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    )
}

/// X-Content-Type-Options: nosniff layer.
pub fn x_content_type_options_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue>
{
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    )
}

/// Referrer-Policy layer.
pub fn referrer_policy_layer() -> tower_http::set_header::SetResponseHeaderLayer<HeaderValue> {
    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
        axum::http::header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    )
}
