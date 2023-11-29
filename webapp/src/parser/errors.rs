use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("no proxy found")]
    NoProxyAvailable,
    #[error("failed to build reqwest client")]
    FailedClient(#[from] reqwest::Error),
    #[error("time outed request")]
    RequestTimeOut,
    #[error("scrapper selector error")]
    CrawlerSelectorError,
    #[error("tokio task error: {0}")]
    TokioTaskError(#[from] tokio::task::JoinError),
    #[error("failed to update proxies")]
    FailedToUpdateProxies,
    #[error("failed to find proxy table")]
    FailedToFindProxyTable,
    #[error("failed to find proxy parsing rules")]
    ProxyParsingRulesError,
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("url parsing error {0}")]
    UrlParsingError(#[from] url::ParseError),
    #[error("time duration error")]
    TimeDurationConversionRange(#[from] time::error::ConversionRange),
    #[error("no shops available")]
    FailedTOFindShops,
    #[error("failed to find any shop")]
    NoShopsFound,
}
