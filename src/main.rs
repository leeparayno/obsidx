use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use pulldown_cmark::{Event, Parser as MdParser, Tag, TagEnd};
use regex::Regex;
use notify::{RecursiveMode, Watcher, Config};
use rusqlite::{Connection, params};
use serde::Serialize;
use serde_json::json;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, STORED, STRING, TEXT, FAST, Value};
use tantivy::{doc, Index, IndexReader, TantivyDocument, Term};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "obsidx", version, about = "Obsidian vault indexer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize an index directory
    Init {
        #[arg(long)]
        vault: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
    },
    /// Build or update the index
    Index {
        #[arg(long)]
        vault: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = false)]
        incremental: bool,
    },
    /// Search the index
    Search {
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Get a note by path
    Get {
        #[arg(long)]
        path: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = false)]
        json: bool,
        /// Include content in response
        #[arg(long, default_value_t = false)]
        content: bool,
    },
    /// List tags
    Tags {
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Link graph queries
    Links {
        #[arg(long)]
        from: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Backlinks to a note
    Backlinks {
        #[arg(long)]
        to: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Watch vault and incrementally reindex
    Watch {
        #[arg(long)]
        vault: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = 500)]
        debounce_ms: u64,
    },
    /// Build embeddings index (SQLite)
    EmbedIndex {
        #[arg(long)]
        vault: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = 1500)]
        max_chars: usize,
        #[arg(long, default_value_t = 200)]
        overlap: usize,
    },
    /// Vector search over embeddings
    EmbedSearch {
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Hybrid search (BM25 + Vector) with RRF
    Hybrid {
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long, default_value_t = 60)]
        rrf_k: u32,
        #[arg(long, default_value_t = 50)]
        bm25_limit: usize,
        #[arg(long, default_value_t = 50)]
        vec_limit: usize,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Index stats
    Stats {
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Output JSON schema for CLI responses
    Schema {
        #[arg(long, default_value_t = false)]
        pretty: bool,
    },
    /// Output tool spec for LLM integration
    ToolSpec {
        #[arg(long, default_value_t = false)]
        pretty: bool,
    },
}

#[derive(Debug, Serialize)]
struct SearchResult {
    path: String,
    title: String,
    score: f32,
}

#[derive(Debug, Serialize)]
struct TagCount {
    tag: String,
    count: usize,
}


#[derive(Debug, Serialize)]
struct NoteDetail {
    path: String,
    title: String,
    content: String,
    tags: Vec<String>,
    headings: Vec<String>,
    links: Vec<String>,
    frontmatter: serde_json::Value,
    mtime: i64,
}
#[derive(Debug)]
struct NoteDoc {
    path: String,
    title: String,
    content: String,
    tags: Vec<String>,
    links: Vec<String>,
    headings: Vec<String>,
    frontmatter_json: String,
    mtime: i64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { vault, index } => init_index(&vault, &index),
        Commands::Index {
            vault,
            index,
            incremental,
        } => build_index(&vault, &index, incremental),
        Commands::Search {
            query,
            index,
            limit,
            json,
        } => search_index(&index, &query, limit, json),
        Commands::Get {
            path,
            index,
            json,
            content,
        } => get_note(&index, &path, json, content),
        Commands::Tags { index, json } => list_tags(&index, json),
        Commands::Links { from, index, json } => list_links(&index, &from, json),
        Commands::Backlinks { to, index, json } => list_backlinks(&index, &to, json),
        Commands::Watch { vault, index, debounce_ms } => watch_vault(&vault, &index, debounce_ms),
        Commands::EmbedIndex { vault, index, max_chars, overlap } => embed_index(&vault, &index, max_chars, overlap),
        Commands::EmbedSearch { query, index, limit, json } => embed_search(&index, &query, limit, json),
        Commands::Hybrid { query, index, limit, rrf_k, bm25_limit, vec_limit, json } => hybrid_search(&index, &query, limit, rrf_k, bm25_limit, vec_limit, json),
        Commands::Stats { index, json } => stats(&index, json),
        Commands::Schema { pretty } => print_schema(pretty),
        Commands::ToolSpec { pretty } => print_tool_spec(pretty),
    }
}

fn schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("path", STRING | STORED);
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT);
    schema_builder.add_text_field("tags", TEXT | STORED);
    schema_builder.add_text_field("links", TEXT | STORED);
    schema_builder.add_text_field("links_term", TEXT);
    schema_builder.add_text_field("headings", TEXT | STORED);
    schema_builder.add_text_field("frontmatter", TEXT | STORED);
    schema_builder.add_i64_field("mtime", FAST | STORED);
    schema_builder.build()
}

fn init_index(vault: &str, index_dir: &str) -> Result<()> {
    let index_path = PathBuf::from(index_dir);
    if !index_path.exists() {
        fs::create_dir_all(&index_path)
            .with_context(|| format!("Failed to create index dir: {index_dir}"))?;
    }
    let schema = schema();
    let _index = Index::create_in_dir(&index_path, schema)
        .with_context(|| "Failed to create Tantivy index")?;

    let out = json_response(json!({
        "message": "index initialized",
        "vault": vault,
        "index": index_dir
    }));
    println!("{out}");
    Ok(())
}

fn build_index(vault: &str, index_dir: &str, incremental: bool) -> Result<()> {
    let index_path = PathBuf::from(index_dir);
    if !index_path.exists() {
        fs::create_dir_all(&index_path)
            .with_context(|| format!("Failed to create index dir: {index_dir}"))?;
    }

    let schema = schema();
    let index = if let Ok(idx) = Index::open_in_dir(&index_path) {
        idx
    } else {
        Index::create_in_dir(&index_path, schema.clone())?
    };

    let mut writer = index.writer(50_000_000)?;

    if !incremental {
        writer.delete_all_documents()?;
    }

    let docs = scan_vault(Path::new(vault))?;
    let total_docs = docs.len();
    let fields = schema_fields(&index);

    // Build a quick mtime map for incremental indexing
    let mut existing_mtimes: HashMap<String, i64> = HashMap::new();
    if incremental {
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let schema = index.schema();
        let path_field = schema.get_field("path").unwrap();
        let mtime_field = schema.get_field("mtime").unwrap();
        for segment_reader in searcher.segment_readers() {
            let store_reader = segment_reader.get_store_reader(0)?;
            for doc_id in 0..segment_reader.max_doc() {
                let doc: TantivyDocument = store_reader.get(doc_id)?;
                let path = doc
                    .get_first(path_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mtime = doc
                    .get_first(mtime_field)
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                if !path.is_empty() {
                    existing_mtimes.insert(path, mtime);
                }
            }
        }
    }

    for doc in docs {
        if incremental {
            if let Some(old) = existing_mtimes.get(&doc.path) {
                if *old >= doc.mtime {
                    continue;
                }
            }
            let term = Term::from_field_text(fields.path, &doc.path);
            writer.delete_term(term);
        }

        let mut tdoc = doc! {
            fields.path => doc.path,
            fields.title => doc.title,
            fields.content => doc.content,
            fields.tags => serde_json::to_string(&doc.tags).unwrap_or_else(|_| "[]".to_string()),
            fields.links => serde_json::to_string(&doc.links).unwrap_or_else(|_| "[]".to_string()),
            fields.headings => serde_json::to_string(&doc.headings).unwrap_or_else(|_| "[]".to_string()),
            fields.frontmatter => doc.frontmatter_json,
            fields.mtime => doc.mtime,
        };
        for link in &doc.links {
            tdoc.add_text(fields.links_term, link);
        }
        writer.add_document(tdoc)?;
    }

    writer.commit()?;

    let out = json_response(json!({
        "message": "index built",
        "vault": vault,
        "index": index_dir,
        "documents": total_docs
    }));
    println!("{out}");
    Ok(())
}

fn search_index(index_dir: &str, query: &str, limit: usize, json_out: bool) -> Result<()> {
    let index = Index::open_in_dir(index_dir)
        .with_context(|| format!("Index not found: {index_dir}"))?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let schema = index.schema();
    let path_field = schema.get_field("path").unwrap();
    let title_field = schema.get_field("title").unwrap();
    let content_field = schema.get_field("content").unwrap();
    let tags_field = schema.get_field("tags").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title_field, content_field, tags_field]);
    let q = query_parser.parse_query(query)?;
    let top_docs = searcher.search(&q, &TopDocs::with_limit(limit))?;

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved: TantivyDocument = searcher.doc(doc_address)?;
        let path = retrieved
            .get_first(path_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let title = retrieved
            .get_first(title_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        results.push(SearchResult { path, title, score });
    }

    if json_out {
        let out = json_response(json!({
            "query": query,
            "results": results
        }));
        println!("{out}");
    } else {
        for r in results {
            println!("{}\t{}\t{:.2}", r.path, r.title, r.score);
        }
    }

    Ok(())
}

fn get_note(index_dir: &str, path: &str, json_out: bool, include_content: bool) -> Result<()> {
    let index = Index::open_in_dir(index_dir)
        .with_context(|| format!("Index not found: {index_dir}"))?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = index.schema();
    let path_field = schema.get_field("path").unwrap();

    let term = Term::from_field_text(path_field, path);
    let doc_opt: Option<TantivyDocument> = searcher
        .search(
            &tantivy::query::TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic),
            &TopDocs::with_limit(1),
        )?
        .into_iter()
        .next()
        .map(|(_, addr)| searcher.doc(addr))
        .transpose()?;

    if let Some(doc) = doc_opt {
        let title = doc
            .get_first(schema.get_field("title").unwrap())
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tags = doc
            .get_first(schema.get_field("tags").unwrap())
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            .unwrap_or_default();
        let headings = doc
            .get_first(schema.get_field("headings").unwrap())
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            .unwrap_or_default();
        let links = doc
            .get_first(schema.get_field("links").unwrap())
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            .unwrap_or_default();
        let frontmatter = doc
            .get_first(schema.get_field("frontmatter").unwrap())
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .unwrap_or_else(|| json!({}));
        let mtime = doc
            .get_first(schema.get_field("mtime").unwrap())
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let content = if include_content {
            doc.get_first(schema.get_field("content").unwrap())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            "".to_string()
        };

        let detail = NoteDetail {
            path: path.to_string(),
            title: title.to_string(),
            content,
            tags,
            headings,
            links,
            frontmatter,
            mtime,
        };

        if json_out {
            let out = json_response(json!({ "note": detail }));
            println!("{out}");
        } else {
            println!("{}\t{}", detail.path, detail.title);
        }
    } else if json_out {
        let out = json_response(json!({
            "error": {
                "code": "not_found",
                "message": format!("No note found for path: {path}")
            }
        }));
        println!("{out}");
    }

    Ok(())
}

fn list_tags(index_dir: &str, json_out: bool) -> Result<()> {
    let index = Index::open_in_dir(index_dir)
        .with_context(|| format!("Index not found: {index_dir}"))?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = index.schema();
    let tags_field = schema.get_field("tags").unwrap();

    let mut counts: HashMap<String, usize> = HashMap::new();
    for segment_reader in searcher.segment_readers() {
        let store_reader = segment_reader.get_store_reader(0)?;
        for doc_id in 0..segment_reader.max_doc() {
            let doc: TantivyDocument = store_reader.get(doc_id)?;
            if let Some(val) = doc.get_first(tags_field).and_then(|v| v.as_str()) {
                if let Ok(tags) = serde_json::from_str::<Vec<String>>(val) {
                    for tag in tags {
                        *counts.entry(tag).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    let mut results: Vec<TagCount> = counts
        .into_iter()
        .map(|(tag, count)| TagCount { tag, count })
        .collect();
    results.sort_by(|a, b| b.count.cmp(&a.count));

    if json_out {
        let out = json_response(json!({ "results": results }));
        println!("{out}");
    } else {
        for r in results {
            println!("{}\t{}", r.tag, r.count);
        }
    }

    Ok(())
}

fn list_links(index_dir: &str, from: &str, json_out: bool) -> Result<()> {
    let index = Index::open_in_dir(index_dir)
        .with_context(|| format!("Index not found: {index_dir}"))?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = index.schema();
    let path_field = schema.get_field("path").unwrap();
    let links_field = schema.get_field("links").unwrap();

    let term = Term::from_field_text(path_field, from);
    let doc_opt: Option<TantivyDocument> = searcher
        .search(
            &tantivy::query::TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic),
            &TopDocs::with_limit(1),
        )?
        .into_iter()
        .next()
        .map(|(_, addr)| searcher.doc(addr))
        .transpose()?;

    let mut links: Vec<String> = vec![];
    if let Some(doc) = doc_opt {
        if let Some(val) = doc.get_first(links_field).and_then(|v| v.as_str()) {
            links = serde_json::from_str::<Vec<String>>(val).unwrap_or_default();
        }
    }

    if json_out {
        let out = json_response(json!({ "from": from, "links": links }));
        println!("{out}");
    } else {
        for l in links {
            println!("{l}");
        }
    }

    Ok(())
}


fn list_backlinks(index_dir: &str, to: &str, json_out: bool) -> Result<()> {
    let index = Index::open_in_dir(index_dir)
        .with_context(|| format!("Index not found: {index_dir}"))?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = index.schema();
    let path_field = schema.get_field("path").unwrap();
    let links_term_field = schema.get_field("links_term").unwrap();

    let term = Term::from_field_text(links_term_field, to);
    let q = tantivy::query::TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
    let top_docs = searcher.search(&q, &TopDocs::with_limit(10_000))?;

    let mut results: Vec<String> = Vec::new();
    for (_score, doc_address) in top_docs {
        let doc: TantivyDocument = searcher.doc(doc_address)?;
        let path = doc
            .get_first(path_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if !path.is_empty() {
            results.push(path);
        }
    }

    results.sort();
    results.dedup();

    if json_out {
        let out = json_response(json!({ "to": to, "backlinks": results }));
        println!("{out}");
    } else {
        for r in results {
            println!("{r}");
        }
    }

    Ok(())
}

fn watch_vault(vault: &str, index_dir: &str, debounce_ms: u64) -> Result<()> {
    // Initial index
    build_index(vault, index_dir, true)?;

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.configure(Config::default().with_poll_interval(Duration::from_millis(250)))?;
    watcher.watch(Path::new(vault), RecursiveMode::Recursive)?;

    println!("Watching {} (index: {})", vault, index_dir);

    loop {
        // block until event
        let _ = rx.recv();
        // debounce: drain events for debounce_ms
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_millis(debounce_ms) {
            if rx.try_recv().is_err() {
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        // incremental rebuild
        let _ = build_index(vault, index_dir, true);
    }
}


#[derive(Debug, Serialize)]
struct VectorResult {
    path: String,
    score: f32,
    chunk: String,
}

fn embed_index(vault: &str, index_dir: &str, max_chars: usize, overlap: usize) -> Result<()> {
    fs::create_dir_all(index_dir).ok();
    let db_path = Path::new(index_dir).join("embeddings.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS chunks (            id INTEGER PRIMARY KEY,            path TEXT,            chunk TEXT,            mtime INTEGER,            embedding TEXT        );",
    )?;

    // Full rebuild for now
    conn.execute("DELETE FROM chunks", [])?;

    let docs = scan_vault(Path::new(vault))?;
    let mut inserted = 0;
    for doc in docs {
        let chunks = chunk_text(&doc.content, max_chars, overlap);
        for ch in chunks {
            let emb = hash_embedding(&ch, 256);
            let emb_json = serde_json::to_string(&emb).unwrap_or_else(|_| "[]".to_string());
            conn.execute(
                "INSERT INTO chunks (path, chunk, mtime, embedding) VALUES (?1, ?2, ?3, ?4)",
                params![doc.path, ch, doc.mtime, emb_json],
            )?;
            inserted += 1;
        }
    }

    let out = json_response(json!({
        "message": "embeddings indexed (hash placeholder)",
        "vault": vault,
        "index": index_dir,
        "chunks": inserted
    }));
    println!("{out}");
    Ok(())
}

fn embed_search(index_dir: &str, query: &str, limit: usize, json_out: bool) -> Result<()> {
    let db_path = Path::new(index_dir).join("embeddings.db");
    let conn = Connection::open(db_path)?;
    let qemb = hash_embedding(query, 256);

    let mut stmt = conn.prepare("SELECT path, chunk, embedding FROM chunks")?;
    let rows = stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let chunk: String = row.get(1)?;
        let emb_json: String = row.get(2)?;
        let emb: Vec<f32> = serde_json::from_str(&emb_json).unwrap_or_default();
        Ok((path, chunk, emb))
    })?;

    let mut results: Vec<VectorResult> = Vec::new();
    for row in rows {
        let (path, chunk, emb) = row?;
        let score = cosine_sim(&qemb, &emb);
        results.push(VectorResult { path, score, chunk });
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results.truncate(limit);

    if json_out {
        let out = json_response(json!({ "query": query, "results": results }));
        println!("{out}");
    } else {
        for r in results {
            println!("{}	{:.3}	{}", r.path, r.score, r.chunk);
        }
    }
    Ok(())
}

fn hybrid_search(index_dir: &str, query: &str, limit: usize, rrf_k: u32, bm25_limit: usize, vec_limit: usize, json_out: bool) -> Result<()> {
    let bm25 = bm25_search(index_dir, query, bm25_limit)?;
    let vec = embed_search_results(index_dir, query, vec_limit)?;

    let mut scores: HashMap<String, f32> = HashMap::new();

    for (rank, item) in bm25.iter().enumerate() {
        let r = (rrf_k + (rank as u32) + 1) as f32;
        *scores.entry(item.path.clone()).or_insert(0.0) += 1.0 / r;
    }
    for (rank, item) in vec.iter().enumerate() {
        let r = (rrf_k + (rank as u32) + 1) as f32;
        *scores.entry(item.path.clone()).or_insert(0.0) += 1.0 / r;
    }

    let mut fused: Vec<(String, f32)> = scores.into_iter().collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    fused.truncate(limit);

    if json_out {
        let out = json_response(json!({ "query": query, "results": fused }));
        println!("{out}");
    } else {
        for (path, score) in fused {
            println!("{}	{:.4}", path, score);
        }
    }

    Ok(())
}

fn bm25_search(index_dir: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let index = Index::open_in_dir(index_dir)
        .with_context(|| format!("Index not found: {index_dir}"))?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let schema = index.schema();
    let path_field = schema.get_field("path").unwrap();
    let title_field = schema.get_field("title").unwrap();
    let content_field = schema.get_field("content").unwrap();
    let tags_field = schema.get_field("tags").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title_field, content_field, tags_field]);
    let q = query_parser.parse_query(query)?;
    let top_docs = searcher.search(&q, &TopDocs::with_limit(limit))?;

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved: TantivyDocument = searcher.doc(doc_address)?;
        let path = retrieved
            .get_first(path_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let title = retrieved
            .get_first(title_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        results.push(SearchResult { path, title, score });
    }
    Ok(results)
}

fn embed_search_results(index_dir: &str, query: &str, limit: usize) -> Result<Vec<VectorResult>> {
    let db_path = Path::new(index_dir).join("embeddings.db");
    let conn = Connection::open(db_path)?;
    let qemb = hash_embedding(query, 256);

    let mut stmt = conn.prepare("SELECT path, chunk, embedding FROM chunks")?;
    let rows = stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let chunk: String = row.get(1)?;
        let emb_json: String = row.get(2)?;
        let emb: Vec<f32> = serde_json::from_str(&emb_json).unwrap_or_default();
        Ok((path, chunk, emb))
    })?;

    let mut results: Vec<VectorResult> = Vec::new();
    for row in rows {
        let (path, chunk, emb) = row?;
        let score = cosine_sim(&qemb, &emb);
        results.push(VectorResult { path, score, chunk });
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results.truncate(limit);
    Ok(results)
}

fn chunk_text(text: &str, max_chars: usize, overlap: usize) -> Vec<String> {
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let end = usize::min(start + max_chars, text.len());
        let chunk = text[start..end].to_string();
        chunks.push(chunk);
        if end == text.len() { break; }
        start = end.saturating_sub(overlap);
    }
    chunks
}

fn hash_embedding(text: &str, dims: usize) -> Vec<f32> {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut vec = vec![0f32; dims];
    for (i, ch) in text.chars().enumerate() {
        let mut h = DefaultHasher::new();
        ch.hash(&mut h);
        let idx = (h.finish() as usize + i) % dims;
        vec[idx] += 1.0;
    }
    let norm = (vec.iter().map(|v| v*v).sum::<f32>()).sqrt();
    if norm > 0.0 {
        for v in &mut vec { *v /= norm; }
    }
    vec
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() { return 0.0; }
    let mut dot = 0.0; let mut na = 0.0; let mut nb = 0.0;
    for i in 0..a.len() {
        dot += a[i]*b[i];
        na += a[i]*a[i];
        nb += b[i]*b[i];
    }
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na.sqrt()*nb.sqrt()) }
}

fn stats(index_dir: &str, json_out: bool) -> Result<()> {
    let index = Index::open_in_dir(index_dir)
        .with_context(|| format!("Index not found: {index_dir}"))?;
    let reader: IndexReader = index.reader()?;
    let searcher = reader.searcher();

    let num_docs = searcher.num_docs();
    let out = json_response(json!({ "documents": num_docs }));

    if json_out {
        println!("{out}");
    } else {
        println!("{num_docs}");
    }
    Ok(())
}

struct SchemaFields {
    path: Field,
    title: Field,
    content: Field,
    tags: Field,
    links: Field,
    links_term: Field,
    headings: Field,
    frontmatter: Field,
    mtime: Field,
}

fn schema_fields(index: &Index) -> SchemaFields {
    let schema = index.schema();
    SchemaFields {
        path: schema.get_field("path").unwrap(),
        title: schema.get_field("title").unwrap(),
        content: schema.get_field("content").unwrap(),
        tags: schema.get_field("tags").unwrap(),
        links: schema.get_field("links").unwrap(),
        links_term: schema.get_field("links_term").unwrap(),
        headings: schema.get_field("headings").unwrap(),
        frontmatter: schema.get_field("frontmatter").unwrap(),
        mtime: schema.get_field("mtime").unwrap(),
    }
}

fn scan_vault(vault: &Path) -> Result<Vec<NoteDoc>> {
    let mut docs = Vec::new();
    for entry in WalkDir::new(vault).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed reading: {}", path.display()))?;
            let meta = fs::metadata(path)?;
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let parsed = parse_note(path, &content);
            docs.push(NoteDoc {
                path: path.to_string_lossy().to_string(),
                title: parsed.title,
                content: parsed.content,
                tags: parsed.tags,
                links: parsed.links,
                headings: parsed.headings,
                frontmatter_json: parsed.frontmatter_json,
                mtime,
            });
        }
    }
    Ok(docs)
}

struct ParsedNote {
    title: String,
    content: String,
    tags: Vec<String>,
    links: Vec<String>,
    headings: Vec<String>,
    frontmatter_json: String,
}

fn parse_note(path: &Path, raw: &str) -> ParsedNote {
    let (frontmatter_raw, body) = extract_frontmatter(raw);
    let mut tags = extract_inline_tags(&body);

    let frontmatter_json = if let Some(raw_fm) = frontmatter_raw.as_deref() {
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(raw_fm) {
            if let Some(fm_tags) = extract_yaml_tags(&yaml) {
                tags.extend(fm_tags);
            }
            serde_json::to_string(&yaml).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        }
    } else {
        "{}".to_string()
    };

    tags.sort();
    tags.dedup();

    let (headings, links) = extract_headings_and_links(&body);

    let title = headings
        .first()
        .map(|h| h.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

    ParsedNote {
        title,
        content: body,
        tags,
        links,
        headings,
        frontmatter_json,
    }
}

fn extract_frontmatter(raw: &str) -> (Option<String>, String) {
    if raw.starts_with("---\n") {
        if let Some(end) = raw[4..].find("\n---") {
            let fm = &raw[4..4 + end];
            let rest = &raw[4 + end + 4..];
            return (Some(fm.to_string()), rest.trim_start().to_string());
        }
    }
    (None, raw.to_string())
}

fn extract_yaml_tags(yaml: &serde_yaml::Value) -> Option<Vec<String>> {
    match yaml.get("tags") {
        Some(serde_yaml::Value::Sequence(seq)) => {
            let tags = seq
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();
            Some(tags)
        }
        Some(serde_yaml::Value::String(s)) => Some(vec![s.to_string()]),
        _ => None,
    }
}

fn extract_inline_tags(body: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)(?:^|\s)#([A-Za-z0-9_\-/]+)").unwrap();
    re.captures_iter(body)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect()
}

fn extract_headings_and_links(body: &str) -> (Vec<String>, Vec<String>) {
    let parser = MdParser::new(body);
    let mut headings = Vec::new();
    let mut links = Vec::new();

    let mut in_heading = false;
    let mut heading_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                in_heading = true;
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if !heading_text.is_empty() {
                    headings.push(heading_text.trim().to_string());
                }
                in_heading = false;
            }
            Event::Text(t) => {
                if in_heading {
                    heading_text.push_str(&t);
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                links.push(dest_url.to_string());
            }
            _ => {}
        }
    }

    // Wikilinks [[note]]
    let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    for cap in re.captures_iter(body) {
        if let Some(m) = cap.get(1) {
            links.push(m.as_str().to_string());
        }
    }

    links.sort();
    links.dedup();

    (headings, links)
}

fn json_response(payload: serde_json::Value) -> String {
    let wrapper = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
        "data": payload
    });
    serde_json::to_string_pretty(&wrapper).unwrap_or_else(|_| "{}".to_string())
}

fn print_schema(pretty: bool) -> Result<()> {
    let schema = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "response": {
            "version": "string",
            "timestamp": "RFC3339 string",
            "data": "object"
        },
        "commands": {
            "search": {"data": {"query": "string", "results": [{"path": "string", "title": "string", "score": "float"}] }},
            "get": {"data": {"path": "string", "title": "string", "tags": ["string"], "headings": ["string"], "links": ["string"], "frontmatter": "object", "mtime": "int", "content": "string"}},
            "tags": {"data": {"results": [{"tag": "string", "count": "int"}]}},
            "links": {"data": {"from": "string", "links": ["string"]}},
            "backlinks": {"data": {"to": "string", "backlinks": ["string"]}},
            "stats": {"data": {"documents": "int"}},
            "init/index": {"data": {"message": "string", "vault": "string", "index": "string", "documents": "int"}}
        }
    });
    let out = if pretty { serde_json::to_string_pretty(&schema)? } else { serde_json::to_string(&schema)? };
    println!("{out}");
    Ok(())
}

fn print_tool_spec(pretty: bool) -> Result<()> {
    let spec = json!({
        "name": "obsidx",
        "description": "Local Obsidian vault indexer with JSON output. Composable CLI for LLM tools.",
        "commands": [
            {"name": "init", "args": "--vault <path> --index <path>", "json": true},
            {"name": "index", "args": "--vault <path> --index <path> [--incremental]", "json": true},
            {"name": "search", "args": "--index <path> --query <q> --limit 20 --json", "json": true},
            {"name": "get", "args": "--index <path> --path <note.md> --json [--content]", "json": true},
            {"name": "tags", "args": "--index <path> --json", "json": true},
            {"name": "links", "args": "--index <path> --from <note.md> --json", "json": true},
            {"name": "backlinks", "args": "--index <path> --to <note.md> --json", "json": true},
            {"name": "watch", "args": "--vault <path> --index <path> --debounce-ms 500", "json": false},
            {"name": "stats", "args": "--index <path> --json", "json": true}
        ],
        "output_contract": "All --json commands return {version, timestamp, data} with stable schemas.",
        "errors": "On failure, return data.error = {code, message} where possible."
    });
    let out = if pretty { serde_json::to_string_pretty(&spec)? } else { serde_json::to_string(&spec)? };
    println!("{out}");
    Ok(())
}
