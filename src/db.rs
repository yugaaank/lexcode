use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{SearchCandidate, SearchResult, SeedFile};
use crate::sync;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open_default() -> Result<Self, Box<dyn std::error::Error>> {
        let path = default_data_dir()?.join("codelex.db");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let database = Self {
            connection: Connection::open(path)?,
        };
        database.migrate()?;
        database.seed_if_empty()?;
        let _ = database.sync_packs();
        Ok(database)
    }

    pub fn open_memory_seeded() -> Result<Self, Box<dyn std::error::Error>> {
        let database = Self {
            connection: Connection::open_in_memory()?,
        };
        database.migrate()?;
        database.seed_if_empty()?;
        Ok(database)
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn migrate(&self) -> rusqlite::Result<()> {
        self.connection.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS languages (
              id INTEGER PRIMARY KEY,
              name TEXT UNIQUE NOT NULL,
              alias TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS topics (
              id INTEGER PRIMARY KEY,
              language_id INTEGER NOT NULL,
              title TEXT NOT NULL,
              category TEXT NOT NULL,
              UNIQUE(language_id, title),
              FOREIGN KEY(language_id) REFERENCES languages(id)
            );

            CREATE TABLE IF NOT EXISTS snippets (
              id INTEGER PRIMARY KEY,
              topic_id INTEGER NOT NULL,
              snippet TEXT NOT NULL,
              explanation TEXT,
              priority INTEGER NOT NULL DEFAULT 0,
              FOREIGN KEY(topic_id) REFERENCES topics(id)
            );

            CREATE TABLE IF NOT EXISTS aliases (
              id INTEGER PRIMARY KEY,
              topic_id INTEGER NOT NULL,
              alias TEXT NOT NULL,
              UNIQUE(topic_id, alias),
              FOREIGN KEY(topic_id) REFERENCES topics(id)
            );

            CREATE TABLE IF NOT EXISTS related_topics (
              topic_id INTEGER NOT NULL,
              related_topic_id INTEGER NOT NULL,
              UNIQUE(topic_id, related_topic_id),
              FOREIGN KEY(topic_id) REFERENCES topics(id),
              FOREIGN KEY(related_topic_id) REFERENCES topics(id)
            );

            CREATE TABLE IF NOT EXISTS sessions (
              id INTEGER PRIMARY KEY,
              name TEXT UNIQUE NOT NULL,
              language TEXT NOT NULL,
              created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS session_queries (
              session_id INTEGER NOT NULL,
              query TEXT NOT NULL,
              timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
              FOREIGN KEY(session_id) REFERENCES sessions(id)
            );

            CREATE TABLE IF NOT EXISTS app_state (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS bookmarks (
              id INTEGER PRIMARY KEY,
              session_id INTEGER,
              language TEXT NOT NULL,
              topic TEXT NOT NULL,
              created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
              UNIQUE(session_id, language, topic),
              FOREIGN KEY(session_id) REFERENCES sessions(id)
            );

            CREATE TABLE IF NOT EXISTS pins (
              id INTEGER PRIMARY KEY,
              session_id INTEGER,
              language TEXT NOT NULL,
              topic TEXT NOT NULL,
              created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
              UNIQUE(session_id, language, topic),
              FOREIGN KEY(session_id) REFERENCES sessions(id)
            );

            CREATE INDEX IF NOT EXISTS idx_languages_name ON languages(name);
            CREATE INDEX IF NOT EXISTS idx_topics_title ON topics(title);
            CREATE INDEX IF NOT EXISTS idx_aliases_alias ON aliases(alias);
            CREATE INDEX IF NOT EXISTS idx_snippets_topic ON snippets(topic_id);
            CREATE INDEX IF NOT EXISTS idx_session_queries_session ON session_queries(session_id);
            CREATE INDEX IF NOT EXISTS idx_bookmarks_session ON bookmarks(session_id);
            CREATE INDEX IF NOT EXISTS idx_pins_session ON pins(session_id);
            ",
        )
    }

    pub fn seed_if_empty(&self) -> Result<(), Box<dyn std::error::Error>> {
        let seed: SeedFile = serde_json::from_str(include_str!("../data/snippets.json"))?;
        validate_seed(&seed)?;

        for language in seed.languages {
            let primary_alias = language
                .aliases
                .first()
                .cloned()
                .unwrap_or_else(|| language.name.clone());
            self.connection.execute(
                "INSERT OR IGNORE INTO languages (name, alias) VALUES (?1, ?2)",
                params![language.name, primary_alias],
            )?;
            let language_id = self.language_id(&language.name)?;

            for topic in language.topics {
                self.connection.execute(
                    "INSERT OR IGNORE INTO topics (language_id, title, category) VALUES (?1, ?2, ?3)",
                    params![language_id, topic.title, topic.category],
                )?;
                let topic_id = self
                    .topic_id(&language.name, &topic.title)?
                    .ok_or_else(|| {
                        format!("failed to create topic '{}:{}'", language.name, topic.title)
                    })?;

                for (priority, snippet) in topic.snippets.into_iter().enumerate() {
                    self.connection.execute(
                        "
                        INSERT INTO snippets (topic_id, snippet, priority)
                        SELECT ?1, ?2, ?3
                        WHERE NOT EXISTS (
                          SELECT 1 FROM snippets WHERE topic_id = ?1 AND snippet = ?2
                        )
                        ",
                        params![topic_id, snippet, priority as i64],
                    )?;
                }

                for alias in topic.aliases {
                    self.connection.execute(
                        "INSERT OR IGNORE INTO aliases (topic_id, alias) VALUES (?1, ?2)",
                        params![topic_id, alias],
                    )?;
                }
            }
        }

        self.seed_extra_topics()?;
        self.link_related_topics()?;
        Ok(())
    }

    pub fn resolve_language(&self, value: &str) -> rusqlite::Result<Option<String>> {
        self.connection
            .query_row(
                "SELECT name FROM languages WHERE name = ?1 OR alias = ?1",
                params![value],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn list_languages(&self) -> rusqlite::Result<Vec<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT name FROM languages ORDER BY name")?;
        statement
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()
    }

    pub fn count_snippets(&self) -> rusqlite::Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM snippets", [], |row| row.get(0))
    }

    pub fn all_candidates(&self) -> rusqlite::Result<Vec<SearchCandidate>> {
        let mut statement = self.connection.prepare(
            "
            SELECT t.id, l.name, t.title, t.category
            FROM topics t
            JOIN languages l ON l.id = t.language_id
            ORDER BY l.name, t.title
            ",
        )?;

        let topics = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        topics
            .into_iter()
            .map(|(topic_id, language, topic, category)| {
                let usage_count = self.usage_count(&language, &topic)?;
                Ok(SearchCandidate {
                    topic_id,
                    aliases: self.aliases_for_topic(topic_id)?,
                    result: SearchResult {
                        language,
                        topic,
                        category,
                        snippets: self.snippets_for(topic_id)?,
                        related: self.related_for(topic_id)?,
                        score: 0,
                    },
                    usage_count,
                })
            })
            .collect()
    }

    pub fn aliases_for_topic(&self, topic_id: i64) -> rusqlite::Result<Vec<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT alias FROM aliases WHERE topic_id = ?1 ORDER BY alias")?;
        statement
            .query_map(params![topic_id], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()
    }

    pub fn topic_id(&self, language: &str, title: &str) -> rusqlite::Result<Option<i64>> {
        self.connection
            .query_row(
                "
                SELECT t.id
                FROM topics t
                JOIN languages l ON l.id = t.language_id
                WHERE l.name = ?1 AND t.title = ?2
                ",
                params![language, title],
                |row| row.get(0),
            )
            .optional()
    }

    fn language_id(&self, language: &str) -> rusqlite::Result<i64> {
        self.connection.query_row(
            "SELECT id FROM languages WHERE name = ?1",
            params![language],
            |row| row.get(0),
        )
    }

    fn seed_extra_topics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let topics = extra_seed_topics();
        for (language, aliases, title, category, topic_aliases, snippet) in topics {
            let primary_alias = aliases.first().copied().unwrap_or(language);
            self.connection.execute(
                "INSERT OR IGNORE INTO languages (name, alias) VALUES (?1, ?2)",
                params![language, primary_alias],
            )?;
            let language_id = self.language_id(language)?;
            self.connection.execute(
                "INSERT OR IGNORE INTO topics (language_id, title, category) VALUES (?1, ?2, ?3)",
                params![language_id, title, category],
            )?;
            let topic_id = self
                .topic_id(language, title)?
                .ok_or_else(|| format!("failed to create extra topic '{language}:{title}'"))?;
            self.connection.execute(
                "
                INSERT INTO snippets (topic_id, snippet, priority)
                SELECT ?1, ?2, 0
                WHERE NOT EXISTS (
                  SELECT 1 FROM snippets WHERE topic_id = ?1 AND snippet = ?2
                )
                ",
                params![topic_id, snippet],
            )?;
            for alias in topic_aliases {
                self.connection.execute(
                    "INSERT OR IGNORE INTO aliases (topic_id, alias) VALUES (?1, ?2)",
                    params![topic_id, alias],
                )?;
            }
        }
        Ok(())
    }

    fn snippets_for(&self, topic_id: i64) -> rusqlite::Result<Vec<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT snippet FROM snippets WHERE topic_id = ?1 ORDER BY priority")?;
        statement
            .query_map(params![topic_id], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()
    }

    fn related_for(&self, topic_id: i64) -> rusqlite::Result<Vec<String>> {
        let mut statement = self.connection.prepare(
            "
            SELECT rt.title
            FROM related_topics rel
            JOIN topics rt ON rt.id = rel.related_topic_id
            WHERE rel.topic_id = ?1
            ORDER BY rt.title
            ",
        )?;
        statement
            .query_map(params![topic_id], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()
    }

    fn usage_count(&self, language: &str, topic: &str) -> rusqlite::Result<i64> {
        self.connection.query_row(
            "
            SELECT COUNT(*)
            FROM session_queries sq
            JOIN sessions s ON s.id = sq.session_id
            WHERE s.language = ?1 AND sq.query LIKE '%' || ?2 || '%'
            ",
            params![language, topic],
            |row| row.get(0),
        )
    }

    pub fn link_related_topics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let seed: SeedFile = serde_json::from_str(include_str!("../data/snippets.json"))?;
        for language in seed.languages {
            for topic in language.topics {
                let Some(topic_id) = self.topic_id(&language.name, &topic.title)? else {
                    continue;
                };
                for related in topic.related {
                    if let Some(related_id) = self.topic_id(&language.name, &related)? {
                        self.connection.execute(
                            "INSERT OR IGNORE INTO related_topics (topic_id, related_topic_id) VALUES (?1, ?2)",
                            params![topic_id, related_id],
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn sync_packs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let packs_dir = default_packs_dir()?;
        if !packs_dir.exists() {
            fs::create_dir_all(&packs_dir)?;
            return Ok(());
        }

        let snippets = sync::parse_pack(&packs_dir)?;
        for ps in snippets {
            // 1. Ensure language exists
            self.connection.execute(
                "INSERT OR IGNORE INTO languages (name, alias) VALUES (?1, ?1)",
                params![ps.language],
            )?;
            let language_id = self.language_id(&ps.language)?;

            // 2. Ensure topic exists
            self.connection.execute(
                "INSERT OR IGNORE INTO topics (language_id, title, category) VALUES (?1, ?2, ?3)",
                params![language_id, ps.frontmatter.topic, ps.frontmatter.category],
            )?;
            let topic_id = self
                .topic_id(&ps.language, &ps.frontmatter.topic)?
                .unwrap();

            // 3. Upsert snippet
            self.connection.execute(
                "
                INSERT INTO snippets (topic_id, snippet, priority)
                SELECT ?1, ?2, 0
                WHERE NOT EXISTS (
                  SELECT 1 FROM snippets WHERE topic_id = ?1 AND snippet = ?2
                )
                ",
                params![topic_id, ps.snippet],
            )?;

            // 4. Add aliases
            for alias in ps.frontmatter.aliases {
                self.connection.execute(
                    "INSERT OR IGNORE INTO aliases (topic_id, alias) VALUES (?1, ?2)",
                    params![topic_id, alias],
                )?;
            }

            // 5. Add related
            for related in ps.frontmatter.related {
                if let Some(related_id) = self.topic_id(&ps.language, &related)? {
                    self.connection.execute(
                        "INSERT OR IGNORE INTO related_topics (topic_id, related_topic_id) VALUES (?1, ?2)",
                        params![topic_id, related_id],
                    )?;
                }
            }
        }

        Ok(())
    }
}

pub fn default_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(path) = std::env::var("CODELEX_CONFIG_DIR") {
        return Ok(PathBuf::from(path));
    }
    let dirs = ProjectDirs::from("", "", "codelex").ok_or("unable to resolve config directory")?;
    Ok(dirs.config_dir().to_path_buf())
}

pub fn default_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(path) = std::env::var("CODELEX_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    let dirs = ProjectDirs::from("", "", "codelex").ok_or("unable to resolve data directory")?;
    Ok(dirs.data_dir().to_path_buf())
}

pub fn default_packs_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(default_config_dir()?.join("packs"))
}

fn validate_seed(seed: &SeedFile) -> Result<(), Box<dyn std::error::Error>> {
    if seed.languages.is_empty() {
        return Err("seed file must contain at least one language".into());
    }
    for language in &seed.languages {
        if language.name.trim().is_empty() {
            return Err("language name cannot be empty".into());
        }
        if language.topics.is_empty() {
            return Err(format!("language '{}' has no topics", language.name).into());
        }
        for topic in &language.topics {
            if topic.title.trim().is_empty() {
                return Err(format!("language '{}' has an empty topic", language.name).into());
            }
            if topic.snippets.is_empty() {
                return Err(format!("topic '{}' has no snippets", topic.title).into());
            }
        }
    }
    Ok(())
}

type ExtraTopic = (
    &'static str,
    &'static [&'static str],
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static str,
);

fn extra_seed_topics() -> Vec<ExtraTopic> {
    vec![
        (
            "c++",
            &["cpp"],
            "array append",
            "arrays",
            &["vector push", "push array"],
            "items.push_back(value);",
        ),
        (
            "c++",
            &["cpp"],
            "hashmap insert",
            "hashmap",
            &["unordered map insert"],
            "map[key] = value;",
        ),
        (
            "c++",
            &["cpp"],
            "hashmap iterate",
            "iteration",
            &["unordered map iterate"],
            "for (const auto& [key, value] : map) {\n    // use key, value\n}",
        ),
        (
            "c++",
            &["cpp"],
            "sort list",
            "sorting",
            &["vector sort"],
            "std::sort(items.begin(), items.end());",
        ),
        (
            "c++",
            &["cpp"],
            "read file",
            "files",
            &["file read"],
            "std::ifstream file(path);\nstd::string text((std::istreambuf_iterator<char>(file)), {});",
        ),
        (
            "bash",
            &["sh"],
            "array append",
            "arrays",
            &["append array"],
            "items+=(\"$value\")",
        ),
        (
            "bash",
            &["sh"],
            "hashmap insert",
            "hashmap",
            &["assoc array"],
            "declare -A map\nmap[$key]=$value",
        ),
        (
            "bash",
            &["sh"],
            "iteration list",
            "iteration",
            &["loop array"],
            "for value in \"${items[@]}\"; do\n  :\ndone",
        ),
        (
            "bash",
            &["sh"],
            "read file",
            "files",
            &["file read"],
            "text=$(<\"$path\")",
        ),
        (
            "bash",
            &["sh"],
            "regex match",
            "regex",
            &["bash regex"],
            "[[ $text =~ $pattern ]]",
        ),
        (
            "java",
            &["j"],
            "array append",
            "arrays",
            &["list add"],
            "items.add(value);",
        ),
        (
            "java",
            &["j"],
            "hashmap insert",
            "hashmap",
            &["map put"],
            "map.put(key, value);\nvar value = map.get(key);",
        ),
        (
            "java",
            &["j"],
            "hashmap iterate",
            "iteration",
            &["map iterate"],
            "for (var entry : map.entrySet()) {\n    var key = entry.getKey();\n    var value = entry.getValue();\n}",
        ),
        (
            "java",
            &["j"],
            "sort list",
            "sorting",
            &["list sort"],
            "items.sort(Comparator.naturalOrder());",
        ),
        (
            "java",
            &["j"],
            "json load",
            "json",
            &["parse json"],
            "var value = mapper.readValue(text, Type.class);",
        ),
        (
            "kotlin",
            &["kt"],
            "array append",
            "arrays",
            &["mutable list add"],
            "items.add(value)",
        ),
        (
            "kotlin",
            &["kt"],
            "hashmap insert",
            "hashmap",
            &["map set"],
            "map[key] = value",
        ),
        (
            "kotlin",
            &["kt"],
            "hashmap iterate",
            "iteration",
            &["map iterate"],
            "for ((key, value) in map) {\n    // use key, value\n}",
        ),
        (
            "kotlin",
            &["kt"],
            "sort list",
            "sorting",
            &["list sort"],
            "items.sort()",
        ),
        (
            "kotlin",
            &["kt"],
            "json load",
            "json",
            &["parse json"],
            "val value = Json.decodeFromString<T>(text)",
        ),
        (
            "rust",
            &["rs"],
            "http request",
            "networking",
            &["request", "get url"],
            "let body = reqwest::get(url).await?.text().await?;",
        ),
        (
            "python",
            &["py"],
            "environment variable",
            "environment variables",
            &["env var"],
            "value = os.environ.get(\"NAME\")",
        ),
        (
            "go",
            &["golang"],
            "test function",
            "testing",
            &["unit test"],
            "func TestThing(t *testing.T) {\n    t.Fatal(err)\n}",
        ),
        (
            "javascript",
            &["js"],
            "cli parsing",
            "CLI parsing",
            &["argv"],
            "const args = process.argv.slice(2);",
        ),
        (
            "typescript",
            &["ts"],
            "channel concurrency",
            "concurrency",
            &["promise all"],
            "const results = await Promise.all(tasks);",
        ),
    ]
}
