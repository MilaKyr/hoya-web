use crate::db::Database;
use crate::errors::AppErrors;
use crate::parser::positions_parser::PositionsParser;
use crate::parser::proxy_parser::ProxyManager;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub positions_parser: PositionsParser,
    pub proxy_parser: ProxyManager,
    pub db: Arc<Database>,
}

impl AppState {
    pub fn init(db: Database) -> Self {
        Self {
            positions_parser: PositionsParser::default(),
            proxy_parser: ProxyManager::default(),
            db: Arc::new(db),
        }
    }

    pub async fn parse(&self) -> Result<(), AppErrors> {
        let (shop, _hoya_positions) = self
            .positions_parser
            .parse(&self.db, &self.proxy_parser)
            .await?;
        self.db.push_shop_back(&shop).await;
        Ok(())
    }
}
