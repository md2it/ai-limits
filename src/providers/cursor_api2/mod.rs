mod credentials;
mod fetch;
mod helpers;
mod parse;
mod project;
mod session;
mod transport;

use std::io;

use crate::types::SourceData;

pub use fetch::CursorResponses;
pub use project::build_source_data;

use fetch::fetch_dashboard;
use project::access_denied;

pub fn collect_usage() -> io::Result<SourceData> {
    match fetch_dashboard()? {
        Ok(responses) => Ok(build_source_data(&responses)),
        Err(denied) => Ok(access_denied(denied.message)),
    }
}

pub use session::{
    cache_epoch, collection_epoch, publish_if_current, retain_snapshot, select_source,
    select_source_with,
};
