use std::path::PathBuf;

use anyhow::{Context, Result};
use futures_util::TryStreamExt;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

/// Where a CSV import reads its rows from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Url(String),
    File(PathBuf),
}

impl Source {
    /// Anything that looks like an HTTP(S) URL is fetched, everything else is
    /// treated as a path on the local filesystem.
    pub fn parse(raw: &str) -> Self {
        if raw.starts_with("http://") || raw.starts_with("https://") {
            Source::Url(raw.to_owned())
        } else {
            Source::File(PathBuf::from(raw))
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Source::Url(url) => url.clone(),
            Source::File(path) => path.display().to_string(),
        }
    }

    /// The URL this source is fetched from, or `None` for a file on disk.
    ///
    /// The etag gate only applies to a URL: a local file has no upstream to have
    /// changed, so there is nothing to compare it against.
    pub fn url(&self) -> Option<&str> {
        match self {
            Source::Url(url) => Some(url),
            Source::File(_) => None,
        }
    }

    /// Opens the source as a byte stream. Nothing is buffered up front, so the
    /// caller only ever holds a single chunk of the CSV in memory.
    pub async fn open(&self) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        match self {
            Source::File(path) => {
                let file = tokio::fs::File::open(path)
                    .await
                    .with_context(|| format!("opening {}", path.display()))?;
                Ok(Box::new(file))
            }
            Source::Url(url) => {
                let response = reqwest::Client::new()
                    .get(url)
                    .send()
                    .await
                    .with_context(|| format!("requesting {url}"))?
                    .error_for_status()
                    .with_context(|| format!("requesting {url}"))?;
                let chunks = response.bytes_stream().map_err(std::io::Error::other);
                Ok(Box::new(StreamReader::new(chunks)))
            }
        }
    }
}
