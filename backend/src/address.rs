//! The three ways a premise's address is put to a reader.
//!
//! `apps/web/src/functions/format-address.ts` wrote them for the Next.js app,
//! which spelled them `getFullAddress`, `getOneLineFullAddress` and
//! `getSummaryAddress`. The jobs endpoint and the MCP tools both want them, so
//! they live here rather than in either caller.

use crate::search::Premises;

/// The whole address, one part per line, for a page or a calendar of its own.
///
/// `getFullAddress` in the Next.js app.
pub fn full(premises: &Premises) -> String {
    joined(parts(premises), "\n")
}

/// The whole address on one line, for a sentence about an address.
///
/// `getOneLineFullAddress` in the Next.js app.
pub fn one_line(premises: &Premises) -> String {
    joined(parts(premises), ", ")
}

/// Only the parts that tell one household from another, for a title that has no
/// room for a postcode.
///
/// `getSummaryAddress` in the Next.js app.
pub fn summary(premises: &Premises) -> String {
    joined(household(premises), " ")
}

/// Every part the feed holds for a premise, before the blanks are dropped.
fn parts(premises: &Premises) -> impl Iterator<Item = &str> {
    [
        premises.address_room.as_deref(),
        premises.address_number.as_deref(),
        premises.address_street.as_deref(),
        premises.address_locality.as_deref(),
        premises.address_city.as_deref(),
        premises.address_postcode.as_deref(),
    ]
    .into_iter()
    .flatten()
}

/// The flat or house and the street, which is as much of an address as a reader
/// needs to tell households apart.
fn household(premises: &Premises) -> impl Iterator<Item = &str> {
    [
        premises.address_room.as_deref(),
        premises.address_number.as_deref(),
        premises.address_street.as_deref(),
    ]
    .into_iter()
    .flatten()
}

/// Joins the parts that are actually there. The feed holds blanks and nulls
/// where an address has nothing to say, and a missing part leaves no mark.
fn joined<'a>(parts: impl Iterator<Item = &'a str>, separator: &str) -> String {
    parts
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(separator)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn premises(room: Option<&str>, number: Option<&str>, street: Option<&str>) -> Premises {
        Premises {
            id: 1_000_001,
            address_room: room.map(str::to_string),
            address_number: number.map(str::to_string),
            address_street: street.map(str::to_string),
            address_locality: Some("MEANWOOD".to_string()),
            address_city: Some("LEEDS".to_string()),
            address_postcode: Some("LS6 2SE".to_string()),
            updated_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }

    #[test]
    fn the_full_address_is_one_part_per_line() {
        let premises = premises(Some("FLAT 1"), Some("6"), Some("SHARP MEWS"));

        assert_eq!(
            full(&premises),
            "FLAT 1\n6\nSHARP MEWS\nMEANWOOD\nLEEDS\nLS6 2SE"
        );
    }

    #[test]
    fn the_one_line_address_is_the_same_parts_with_commas() {
        let premises = premises(Some("FLAT 1"), Some("6"), Some("SHARP MEWS"));

        assert_eq!(
            one_line(&premises),
            "FLAT 1, 6, SHARP MEWS, MEANWOOD, LEEDS, LS6 2SE"
        );
    }

    #[test]
    fn the_summary_address_keeps_only_what_tells_households_apart() {
        let premises = premises(Some("FLAT 1"), Some("6"), Some("SHARP MEWS"));

        assert_eq!(summary(&premises), "FLAT 1 6 SHARP MEWS");
    }

    #[test]
    fn an_address_leaves_out_the_parts_that_are_missing_or_empty() {
        let premises = premises(None, None, Some("SHARP MEWS"));

        assert_eq!(full(&premises), "SHARP MEWS\nMEANWOOD\nLEEDS\nLS6 2SE");
        assert_eq!(one_line(&premises), "SHARP MEWS, MEANWOOD, LEEDS, LS6 2SE");
        assert_eq!(summary(&premises), "SHARP MEWS");
    }

    #[test]
    fn an_address_with_nothing_in_it_is_empty() {
        let mut premises = premises(None, None, None);
        premises.address_locality = None;
        premises.address_city = None;
        premises.address_postcode = None;

        assert_eq!(full(&premises), "");
        assert_eq!(summary(&premises), "");
    }
}
