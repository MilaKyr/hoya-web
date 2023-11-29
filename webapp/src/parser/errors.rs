use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("no proxy found")]
    NoProxyAvailable,
    #[error("failed to build reqwest client: {0}")]
    FailedClient(#[from] reqwest::Error),
    #[error("scrapper selector error")]
    CrawlerSelectorError,
    #[error("tokio task error: {0}")]
    TokioTaskError(#[from] tokio::task::JoinError),
    #[error("failed to update proxies")]
    FailedToUpdateProxies,
    #[error("failed to find proxy table")]
    FailedToFindProxyTable,
    #[error("url parsing error {0}")]
    UrlParsingError(#[from] url::ParseError),
    #[error("no shops rules available for {0}")]
    FailedToFindShopsRules(String),
    #[error("failed to find any shop")]
    NoShopsFound,
    #[error("not a proper proxy")]
    NotAProxyRow,
}
