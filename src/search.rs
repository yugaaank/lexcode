use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::db::Database;
use crate::models::{SearchCandidate, SearchResult};

pub fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '+' {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn search(
    database: &Database,
    language: Option<&str>,
    query: &str,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let normalized_query = normalize(query);
    let candidates = database.all_candidates()?;
    let mut results = candidates
        .into_iter()
        .filter_map(|candidate| score_candidate(candidate, language, &normalized_query))
        .collect::<Vec<_>>();

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.language.cmp(&right.language))
            .then_with(|| left.topic.cmp(&right.topic))
    });
    Ok(results)
}

fn score_candidate(
    candidate: SearchCandidate,
    language: Option<&str>,
    normalized_query: &str,
) -> Option<SearchResult> {
    let matcher = SkimMatcherV2::default();
    let topic = normalize(&candidate.result.topic);
    let category = normalize(&candidate.result.category);
    let aliases = candidate
        .aliases
        .iter()
        .map(|alias| normalize(alias))
        .collect::<Vec<_>>();
    let haystack = std::iter::once(topic.as_str())
        .chain(std::iter::once(category.as_str()))
        .chain(aliases.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");

    let mut score = 0;
    if topic == normalized_query || aliases.iter().any(|alias| alias == normalized_query) {
        score += 100;
    }
    if aliases
        .iter()
        .any(|alias| alias.contains(normalized_query) || normalized_query.contains(alias))
    {
        score += 50;
    }
    if tokens_match(normalized_query, &haystack) {
        score += 35;
    }
    if let Some(language) = language
        && candidate.result.language == language
    {
        score += 40;
    }
    if let Some(fuzzy_score) = matcher.fuzzy_match(&haystack, normalized_query) {
        score += fuzzy_score.min(60);
    }
    score += candidate.usage_count.min(20);

    if score == 0 {
        None
    } else {
        Some(SearchResult {
            score,
            ..candidate.result
        })
    }
}

fn tokens_match(query: &str, haystack: &str) -> bool {
    query
        .split_whitespace()
        .all(|token| haystack.split_whitespace().any(|part| part.contains(token)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_symbols_and_case() {
        assert_eq!(normalize("HashMap::Iterate!"), "hashmap iterate");
    }

    #[test]
    fn finds_seeded_rust_topic() {
        let database = Database::open_memory_seeded().unwrap();
        let results = search(&database, Some("rust"), "hashmap iterate").unwrap();
        assert_eq!(results[0].language, "rust");
        assert!(results[0].snippets[0].contains("for (key, value)"));
    }
}
