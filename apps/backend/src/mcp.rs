//! `POST /api/mcp`: the three searches the site makes, offered to MCP clients.
//!
//! The tools and the shape of their answers are fixed, down to the sentences
//! each answer reads as and the same value again as structured content. The
//! protocol itself is left to the SDK.
//!
//! Nothing here is a search of its own: every tool reads through
//! [`crate::search`], the same module the HTTP endpoints use.

use std::cmp::Ordering;

use chrono::{Local, NaiveDate};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::schema_for_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::address;
use crate::search::{self, Premises};

/// Where the site lives, which `get_premises_permalink` points at.
const SITE_URL: &str = "https://bins.felixyeung.com";

/// The tools, served at `POST /api/mcp`.
///
/// Sessions and server sent events are left as the SDK has them, because that is
/// what the handler was serving and clients are written against it. Host
/// checking is off: the endpoint answers for whichever name it is asked for, so
/// the only host a client can use is the one it was already told to trust.
pub fn service(pool: PgPool) -> StreamableHttpService<Bins, LocalSessionManager> {
    StreamableHttpService::new(
        move || Ok(Bins::new(pool.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().disable_allowed_hosts(),
    )
}

/// The server: a pool to read through, and the three tools.
#[derive(Clone)]
pub struct Bins {
    pool: PgPool,
    tool_router: ToolRouter<Bins>,
}

/// One shared pool for every session, so a tool never opens a connection of its
/// own.
impl Bins {
    fn new(pool: PgPool) -> Self {
        Self {
            pool,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl Bins {
    #[tool(
        description = "Search premises or addresses using a postcode. The premises.id can then be used in show_premises_jobs_by_id or get_premises_permalink for their respective functions",
        output_schema = schema_for_output::<Found>(),
        annotations(read_only_hint = true)
    )]
    async fn search_premises_from_postcode(
        &self,
        Parameters(Postcode { postcode }): Parameters<Postcode>,
    ) -> Result<CallToolResult, McpError> {
        let found = search::premises(&self.pool, &postcode)
            .await
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;

        if found.is_empty() {
            return Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "No address matching with postcode {postcode} found"
            ))]));
        }

        let content = found
            .iter()
            .map(|premises| {
                ContentBlock::text(format!("{}: {}", premises.id, address::one_line(premises)))
            })
            .collect();

        answered(content, Found { premises: found })
    }

    #[tool(
        description = "Retrieves a list of bin dates (household waste collection dates) for a given premises or address. This will include the date of collection and the type of bin (e.g. Black, Green etc.)",
        output_schema = schema_for_output::<Collections>(),
        annotations(read_only_hint = true)
    )]
    async fn show_premises_jobs_by_id(
        &self,
        Parameters(PremisesId { premises_id }): Parameters<PremisesId>,
    ) -> Result<CallToolResult, McpError> {
        let Some(premises_id) = as_id(premises_id) else {
            return Ok(unknown(premises_id));
        };

        let Some(jobs) = search::jobs(&self.pool, premises_id)
            .await
            .map_err(|error| McpError::internal_error(error.to_string(), None))?
        else {
            return Ok(unknown(premises_id as u32));
        };

        // The server's own day, rather than a collection's.
        let today = Local::now().date_naive();
        let collections = Collections {
            jobs: jobs
                .jobs
                .iter()
                .map(|job| Collection {
                    bin: job.bin.clone(),
                    date: job.date,
                    status: When::on(job.date, today),
                })
                .collect(),
            premises: jobs.premises,
        };

        let mut content = vec![ContentBlock::text(format!(
            "Full Address:\n{}",
            address::full(&collections.premises)
        ))];

        content.extend(collections.jobs.iter().map(|job| {
            ContentBlock::text(format!("{} ({}): {} bin", job.date, job.status, job.bin))
        }));

        answered(content, collections)
    }

    #[tool(
        description = "Gets the permanent link to a page for this premises or address. The page shows the full address, a simple calendar view for the next few week's collection dates, as well as the full list of bin collection dates by each bin type. The page also includes a iCal link and instructions for users to add to their preferred calendar as an integration",
        output_schema = schema_for_output::<Page>(),
        annotations(read_only_hint = true)
    )]
    async fn get_premises_permalink(
        &self,
        Parameters(PremisesId { premises_id }): Parameters<PremisesId>,
    ) -> Result<CallToolResult, McpError> {
        let Some(premises_id) = as_id(premises_id) else {
            return Ok(unknown(premises_id));
        };

        let known = search::jobs(&self.pool, premises_id)
            .await
            .map_err(|error| McpError::internal_error(error.to_string(), None))?
            .is_some();

        if !known {
            return Ok(unknown(premises_id as u32));
        }

        let page = Page {
            link: format!("{SITE_URL}/premises?id={premises_id}"),
        };

        answered(
            vec![ContentBlock::text(format!(
                "Permalink to the premises page:\n{}",
                page.link
            ))],
            page,
        )
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Bins {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            // The crate is called `backend`; clients show this name instead.
            .with_server_info(
                Implementation::new("bins", env!("CARGO_PKG_VERSION")).with_title("Bins"),
            )
            .with_instructions(
                "Bin collection dates for Leeds, UK. Search for an address by postcode, then use \
             the premises.id it returns with show_premises_jobs_by_id for the collection dates or \
             get_premises_permalink for a page a person can open themselves."
                    .to_string(),
            )
    }
}

/// An answer as a client reads it: the sentences to read, and the same value
/// again as structured content for a client that wants the fields instead.
///
/// The schema for `structured` is declared on each tool, so a client is told
/// what it is being given before it asks.
fn answered<T: Serialize>(
    content: Vec<ContentBlock>,
    structured: T,
) -> Result<CallToolResult, McpError> {
    let mut result = CallToolResult::success(content);

    result.structured_content = Some(
        serde_json::to_value(structured)
            .map_err(|error| McpError::internal_error(error.to_string(), None))?,
    );

    Ok(result)
}

/// An id the site has no premise for, which is also what an id too large to be
/// one of ours amounts to.
fn unknown(premises_id: u32) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(format!(
        "Address not found (bad premisesId of {premises_id})"
    ))])
}

/// The id as the database counts premises, or `None` for a number that is too
/// large to be one.
fn as_id(premises_id: u32) -> Option<i32> {
    i32::try_from(premises_id).ok()
}

/// The premise to look up, as the two tools that take an id take it.
#[derive(Debug, Deserialize, JsonSchema)]
struct PremisesId {
    /// The premises.id returned from search_premises_from_postcode
    #[serde(rename = "premisesId")]
    premises_id: u32,
}

/// The postcode to look for.
#[derive(Debug, Deserialize, JsonSchema)]
struct Postcode {
    /// The UK postcode of the address. For example, LS6 2SE
    postcode: String,
}

/// The premises a postcode search turned up.
#[derive(Debug, Serialize, JsonSchema)]
struct Found {
    premises: Vec<Premises>,
}

/// A premise together with its collections, as `/api/jobs` serves it with where
/// each collection sits added to it.
#[derive(Debug, Serialize, JsonSchema)]
struct Collections {
    #[serde(flatten)]
    premises: Premises,
    jobs: Vec<Collection>,
}

/// One collection, marked with where it sits relative to today.
#[derive(Debug, Serialize, JsonSchema)]
struct Collection {
    bin: String,
    date: NaiveDate,
    status: When,
}

/// The page for one premise.
#[derive(Debug, Serialize, JsonSchema)]
struct Page {
    link: String,
}

/// Where a collection sits relative to today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
enum When {
    Expired,
    Today,
    Upcoming,
}

impl When {
    /// `today` is the day the server thinks it is.
    fn on(date: NaiveDate, today: NaiveDate) -> Self {
        match date.cmp(&today) {
            Ordering::Less => Self::Expired,
            Ordering::Equal => Self::Today,
            Ordering::Greater => Self::Upcoming,
        }
    }
}

impl std::fmt::Display for When {
    /// The names the schema offers a client.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Expired => "Expired",
            Self::Today => "Today",
            Self::Upcoming => "Upcoming",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The date a job list is read against.
    fn day(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("a date")
    }

    #[test]
    fn a_date_is_marked_by_where_it_sits_against_today() {
        let today = day("2026-11-05");

        assert_eq!(When::on(day("2026-11-02"), today), When::Expired);
        assert_eq!(When::on(day("2026-11-05"), today), When::Today);
        assert_eq!(When::on(day("2026-11-09"), today), When::Upcoming);
    }

    #[test]
    fn the_marks_are_the_words_the_next_js_handler_wrote() {
        assert_eq!(When::Expired.to_string(), "Expired");
        assert_eq!(When::Today.to_string(), "Today");
        assert_eq!(When::Upcoming.to_string(), "Upcoming");
    }

    #[test]
    fn the_marks_serialise_as_those_words() {
        assert_eq!(
            serde_json::to_string(&When::Upcoming).expect("a string"),
            "\"Upcoming\""
        );
    }

    #[test]
    fn a_premise_id_is_taken_under_the_name_the_next_js_route_used() {
        let args: PremisesId =
            serde_json::from_value(serde_json::json!({ "premisesId": 42 })).expect("the arguments");

        assert_eq!(args.premises_id, 42);
    }

    #[test]
    fn a_negative_premise_id_is_not_a_premise_id() {
        let arguments = serde_json::json!({ "premisesId": -1 });

        assert!(serde_json::from_value::<PremisesId>(arguments).is_err());
    }

    #[test]
    fn an_id_the_database_could_not_hold_is_no_premise_at_all() {
        assert_eq!(as_id(42), Some(42));
        assert_eq!(as_id(u32::MAX), None);
    }

    /// The schema each tool offers a client, checked against what the tool
    /// actually answers with, since a schema that has drifted is worse than none.
    fn schema_of<T: JsonSchema + std::any::Any>() -> serde_json::Value {
        serde_json::to_value(schema_for_output::<T>()).expect("a schema")
    }

    #[test]
    fn the_collections_schema_is_the_flattened_api_shape_with_a_status() {
        let schema = schema_of::<Collections>();

        assert_eq!(schema["type"], "object");

        let properties = schema["properties"]
            .as_object()
            .expect("the properties of a premise with its collections");

        assert_eq!(
            properties["addressPostcode"]["type"],
            serde_json::json!(["string", "null"]),
            "the address is spread across the top level, and may be missing"
        );
        assert_eq!(properties["jobs"]["type"], serde_json::json!("array"));

        let collection = &schema["$defs"]["Collection"];

        assert_eq!(
            collection["properties"]["status"]["$ref"],
            serde_json::json!("#/$defs/When")
        );
        assert_eq!(
            schema["$defs"]["When"]["enum"],
            serde_json::json!(["Expired", "Today", "Upcoming"]),
            "a client is told the words a status can be"
        );
    }

    #[test]
    fn the_found_schema_is_a_list_of_premises() {
        let schema = schema_of::<Found>();

        assert_eq!(
            schema["properties"]["premises"]["type"],
            serde_json::json!("array")
        );
    }

    #[test]
    fn the_page_schema_is_a_link() {
        let schema = schema_of::<Page>();

        assert_eq!(
            schema["properties"]["link"]["type"],
            serde_json::json!("string")
        );
    }

    #[test]
    fn a_collection_serialises_the_way_the_next_js_handler_wrote_it() {
        let collection = Collection {
            bin: "GREEN".to_string(),
            date: day("2026-11-09"),
            status: When::Upcoming,
        };

        assert_eq!(
            serde_json::to_value(&collection).expect("the collection"),
            serde_json::json!({ "bin": "GREEN", "date": "2026-11-09", "status": "Upcoming" })
        );
    }
}
