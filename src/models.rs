use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Language {
    pub id: i64,
    pub name: String,
    pub alias: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Topic {
    pub id: i64,
    pub language_id: i64,
    pub title: String,
    pub category: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    pub id: i64,
    pub topic_id: i64,
    pub snippet: String,
    pub explanation: Option<String>,
    pub priority: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Alias {
    pub id: i64,
    pub topic_id: i64,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub id: i64,
    pub name: String,
    pub language: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub query: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub language: String,
    pub topic: String,
    pub category: String,
    pub snippets: Vec<String>,
    pub related: Vec<String>,
    pub score: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCandidate {
    #[allow(dead_code)]
    pub topic_id: i64,
    pub result: SearchResult,
    pub aliases: Vec<String>,
    pub usage_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedFile {
    pub languages: Vec<SeedLanguage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedLanguage {
    pub name: String,
    pub aliases: Vec<String>,
    pub topics: Vec<SeedTopic>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedTopic {
    pub title: String,
    pub category: String,
    pub aliases: Vec<String>,
    pub snippets: Vec<String>,
    #[serde(default)]
    pub related: Vec<String>,
}
