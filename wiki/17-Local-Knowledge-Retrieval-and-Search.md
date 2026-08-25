# 17 — Local Knowledge Retrieval & Search

> **Corresponding Specifications:** [`sys-arch/32-search-indexing-local-knowledge-privacy-architecture.md`](../sys-arch/32-search-indexing-local-knowledge-privacy-architecture.md), [`sys-arch/ui-ux-11-search-local-knowledge-retrieval-architecture.md`](../sys-arch/ui-ux-11-search-local-knowledge-retrieval-architecture.md)  
> **Key Crates:** [`crates/siar-storage`](../crates/siar-storage), [`crates/siar-ui-state`](../crates/siar-ui-state)

---

## 1. Zero-Leak Local Knowledge Architecture

Standard cloud messengers index message search queries on remote server clusters, creating massive metadata surveillance risks. In SIAR, search and knowledge retrieval are **100% local, offline, and cryptographically sealed**:

```
[User Search Query: "evacuation route map"]
                    |
                    v
    +---------------+---------------+
    |                               |
    v                               v
[Lexical Index: BM25]       [Semantic Index: Local Embeddings]
(Tokenized Inverted Index)  (Lightweight On-Device Vector Store)
    |                               |
    +---------------+---------------+
                    |
                    v
    [Hybrid Reciprocal Rank Fusion (RRF)]
                    |
                    v
    [Encrypted Result Set Displayed in UI (< 10 ms)]
```

---

## 2. BM25 Lexical Full-Text Search

Messages, documents, notes, and contact metadata are indexed locally using a pure-Rust BM25 engine backed by Stoolap:

```rust
pub struct SearchIndexEntry {
    pub message_id: MessageId,
    pub conversation_id: ConversationId,
    pub author_id: AccountId,
    pub timestamp: Timestamp,
    pub token_terms: Vec<String>,      // Lowercased, stemmed tokens
    pub attachment_mime: Option<String>,
}
```

### Search Features
- **Prefix & Wildcard Matching**: Real-time autocomplete as the user types (`"emer*"` $\rightarrow$ `"emergency"`, `"emergent"`).
- **Faceted Filters**: Narrow results by sender, date range, conversation type (Direct vs Group), and media category (Photos, Audio, PDF Documents).
- **Encrypted Inverted Index**: Index tables are stored in the same AES-256-GCM / ChaCha20 encrypted database container as the message history.

---

## 3. Local Knowledge Retrieval & Offline RAG

For mission-critical emergency teams and off-grid field operations, SIAR supports **Local Knowledge Retrieval**:
- **Offline Field Manuals**: Searchable survival manuals, triage medical guides, and topography maps bundled into local knowledge stores.
- **Smart Retrieval**: Context-aware queries pull up relevant tactical protocols, frequency lists, and emergency contacts instantly even in complete radio silence.
