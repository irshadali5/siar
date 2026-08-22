# Part 32 — Search, Indexing, Local Knowledge Retrieval & Privacy-Preserving Discovery Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 32  
**Primary language:** Rust  
**Primary purpose:** provide fast, private, local-first search and retrieval across messages, files, attachments, contacts, call metadata, plugin-owned data, and optionally local semantic knowledge without weakening end-to-end encryption  
**Primary goals:** incremental local indexing, encrypted/private search state, deterministic rebuilds, offline operation, bounded resource use, fast full-text retrieval, optional semantic/vector retrieval, attachment metadata discovery, message edit/delete handling, multi-device behavior, background indexing, and strict prohibition on server-side plaintext indexing of E2EE data by default

---

# 1. Purpose

A production communication platform eventually contains a large amount of user-owned information:

```text
messages
files
attachments
voice-note metadata
call history
contacts
group names
links
documents
plugin data
ERP events
shared notes
```

Without a dedicated search architecture, users end up with:

```text
slow linear scans
inconsistent results
large memory use
stale indexes
privacy leakage
server-side plaintext search
```

The governing principle is:

> **Search should be local-first, rebuildable, privacy-preserving, and derived from authoritative encrypted/durable state.**

---

# 2. Architectural Position

```text
Authoritative Durable State
        │
        ├── Messages
        ├── Files
        ├── Contacts
        ├── Calls
        └── Plugin data
        │
        ▼
   Indexing Pipeline
        │
        ▼
   Local Search Index
        │
        ▼
  Query / Ranking Engine
        │
        ▼
      Dioxus UI
```

Optional semantic layer:

```text
Local Content
    ↓
Embedding / Feature Extraction
    ↓
Local Vector Index
    ↓
Semantic Retrieval
```

---

# 3. Search Is Derived State

The search index is not authoritative.

If lost or corrupted:

```text
rebuild from durable source
```

This is a major architectural invariant.

---

# 4. Search Domains

```rust
pub enum SearchDomain {
    Messages,
    Files,
    Contacts,
    Calls,
    Groups,
    Links,
    PluginData,
}
```

---

# 5. Search Scope

User may search:

```text
everything
current conversation
one group
one contact
one file type
date range
sender
```

---

# 6. Search Query

```rust
pub struct SearchQuery {
    pub text: String,
    pub scope: SearchScope,
    pub filters: SearchFilters,
    pub limit: usize,
}
```

---

# 7. Search Scope Types

```rust
pub enum SearchScope {
    Global,
    Conversation(ConversationId),
    Group(GroupId),
    Contact(AccountId),
    Domain(SearchDomain),
}
```

---

# 8. Filters

```rust
pub struct SearchFilters {
    pub sender: Option<AccountId>,
    pub after: Option<Timestamp>,
    pub before: Option<Timestamp>,
    pub file_types: Vec<FileType>,
    pub has_attachment: Option<bool>,
}
```

---

# 9. Text Search

Baseline:

```text
tokenized full-text search
prefix search
phrase search
field-aware filtering
```

---

# 10. Local FTS

Use a local embedded index.

The exact backend may be:

```text
SQLite FTS
Tantivy
custom inverted index
```

depending product/DB choices.

Architecture remains backend-neutral.

---

# 11. Search Backend Trait

```rust
pub trait SearchIndex {
    fn upsert(&mut self, doc: SearchDocument) -> Result<(), SearchError>;
    fn delete(&mut self, id: SearchDocumentId) -> Result<(), SearchError>;
    fn query(&self, q: &SearchQuery) -> Result<Vec<SearchHit>, SearchError>;
}
```

---

# 12. Search Document

```rust
pub struct SearchDocument {
    pub id: SearchDocumentId,
    pub domain: SearchDomain,
    pub text: String,
    pub metadata: SearchMetadata,
}
```

---

# 13. Stable Search IDs

Do not use backend row IDs as product identity.

Use logical IDs:

```text
MessageId
BlobId
ConversationId
ContactId
```

wrapped into `SearchDocumentId`.

---

# 14. Message Indexing

Index:

```text
message body
sender display name optionally
attachment names
links
selected metadata
```

Do not index hidden/system secrets.

---

# 15. Edited Messages

When message changes:

```text
same document ID
→ update index
```

No duplicate old result.

---

# 16. Deleted Messages

On deletion:

```text
remove from search index
```

according to actual local deletion semantics.

---

# 17. Expiring Messages

When expired:

```text
delete search entry
```

along with durable content/key erasure policy.

---

# 18. File Indexing

File search may include:

```text
filename
MIME
caption
size
sender
conversation
creation date
```

---

# 19. File Content Search

Optional.

Only index document contents if:

```text
user has local plaintext access
supported file parser exists
resource budget allows
```

---

# 20. File Parsing

Potential:

```text
PDF text
plain text
Markdown
office document text
```

through sandboxed parser modules.

---

# 21. Untrusted File Parsing

Attachments are untrusted input.

Use:

```text
bounded parsers
sandbox/WASM where practical
timeouts
size limits
```

---

# 22. OCR

If future OCR exists:

```text
local OCR
```

can feed index.

Do not require cloud OCR for core functionality.

---

# 23. Voice Notes

Baseline index:

```text
sender
time
duration
caption
```

Optional local transcription later.

---

# 24. Local Transcription

If on-device speech recognition added:

```text
transcript
→ local index
```

User should understand storage/privacy implications.

---

# 25. Calls

Index call metadata only:

```text
participant
date
duration
call type
```

Do not index audio/video unless user created transcription/notes explicitly.

---

# 26. Contacts

Index:

```text
display name
aliases
organization
user-added notes
```

according to privacy policy.

---

# 27. Groups

Index:

```text
group name
description
member names optionally
```

---

# 28. Plugin Data

Plugins can register searchable fields only through explicit API.

They do not get raw access to core search index.

---

# 29. Plugin Search Namespace

```text
PluginId
+
document type
```

---

# 30. Plugin Permission

Potential:

```text
SearchIndexOwnData
```

Default:

```text
own namespace only
```

---

# 31. No Cross-Plugin Search Access

One plugin cannot enumerate another plugin's index.

---

# 32. Search Index Privacy

A plaintext local FTS index may expose message content if the device storage is stolen.

This is a real security trade-off.

---

# 33. Privacy Modes

```rust
pub enum SearchPrivacyMode {
    FastLocal,
    EncryptedIndex,
    RebuildOnUnlock,
}
```

---

# 34. FastLocal

Index stored locally with OS/app storage protection.

Fastest and simplest.

---

# 35. EncryptedIndex

Search index itself encrypted at rest.

Requires backend/architecture support.

---

# 36. RebuildOnUnlock

High-security option:

```text
no persistent plaintext index
```

Rebuild selected searchable state after unlock.

Potentially expensive.

---

# 37. Recommended Default

Use:

```text
local index
+
encrypted device storage / app DB protection
```

and document risk.

Do not send plaintext index to server.

---

# 38. Server-Side Search

Default:

```text
unsupported for E2EE plaintext
```

Server may search only:

```text
non-sensitive public metadata
encrypted opaque IDs
explicit organization-managed data
```

---

# 39. Search Is Not an E2EE Backdoor

Never upload:

```text
message plaintext
tokenized keywords
embeddings
```

to server by default.

Embeddings can leak content semantics.

---

# 40. Semantic Search

Optional local feature.

Examples:

```text
"find the conversation about school fee policy"
"show the file about emergency routing"
```

---

# 41. Semantic Retrieval Architecture

```text
Local plaintext content
      ↓
Local embedding model
      ↓
Vector
      ↓
Local vector index
      ↓
Similarity search
```

---

# 42. Local-Only Embeddings

Default:

```text
generated on device
stored on device
```

---

# 43. Embeddings Are Sensitive

Even though embeddings are not plaintext, they can reveal semantic information.

Treat them as sensitive derived data.

---

# 44. Vector Index Privacy

Protect like:

```text
message content
```

---

# 45. Semantic Search Optionality

Core messenger must not depend on semantic search.

Feature-gate:

```text
semantic-search
```

---

# 46. Model Runtime

Prefer local Rust-compatible inference runtime if viable.

If platform-native accelerator is easier:

```text
narrow adapter
```

is acceptable.

---

# 47. Rust-First Rule

Use Rust for:

```text
chunking
index orchestration
vector storage
ranking
query fusion
```

Platform/native model runtime only if needed.

---

# 48. Embedding Model Version

```rust
pub struct EmbeddingModelVersion(String);
```

Index entries bind to model version.

---

# 49. Model Upgrade

Changing embedding model requires:

```text
re-embedding
```

Do not compare vectors from incompatible models blindly.

---

# 50. Embedding Index Rebuild

Background/rebuildable.

---

# 51. Hybrid Search

Combine:

```text
keyword FTS
+
semantic vector
```

for stronger results.

---

# 52. Ranking

Example:

```text
text relevance
semantic similarity
recency
conversation relevance
sender match
```

---

# 53. Ranking Policy

```rust
pub trait SearchRanker {
    fn rank(
        &self,
        candidates: &mut [SearchHit],
        ctx: &SearchContext,
    );
}
```

---

# 54. Deterministic Baseline Ranking

Start with explainable ranking.

Avoid opaque AI ranker as required dependency.

---

# 55. Search Hit

```rust
pub struct SearchHit {
    pub id: SearchDocumentId,
    pub score: f32,
    pub domain: SearchDomain,
    pub snippet: Option<String>,
}
```

---

# 56. Snippets

Generate locally.

Do not persist giant highlighted snippets.

---

# 57. Highlighting

Query:

```text
architecture
```

result:

```text
"... complete system and architecture ..."
```

---

# 58. Tokenization

Need language-aware tokenization strategy.

Start with robust Unicode tokenization.

---

# 59. Unicode

Handle:

```text
Latin
Devanagari
Arabic
CJK
emoji
mixed scripts
```

without ASCII-only assumptions.

---

# 60. Normalization

Search normalization may include:

```text
case folding
Unicode normalization
```

but message identity/storage remains unchanged.

---

# 61. Accent Handling

Optional accent-insensitive search.

---

# 62. Stemming

Language-specific stemming may improve search.

Do not apply wrong stemmer globally.

---

# 63. Multilingual Search

Store analyzer metadata per document/language if needed.

---

# 64. Language Detection

Optional.

Avoid expensive model if simple heuristics suffice.

---

# 65. Exact Search

Support quoting:

```text
"store carry forward"
```

---

# 66. Prefix Search

Useful for:

```text
contact names
filenames
```

---

# 67. Fuzzy Search

Optional limited edit distance.

Avoid expensive global fuzzy query on giant dataset.

---

# 68. Typo Tolerance

Good for contact/file names.

---

# 69. Filtered Search

Example:

```text
from:Alice has:file after:2026-01-01
```

Product may expose graphical filters instead of query syntax.

---

# 70. Query Parser

If advanced syntax supported:

```text
strict parser
typed filters
```

No arbitrary backend query injection.

---

# 71. Search Query Limits

Bound:

```text
query length
token count
filter count
result limit
```

---

# 72. Search Resource Limits

Part 08 applies:

```text
CPU time
memory
index bytes
parallel queries
background index work
```

---

# 73. Cancellation

Search should be cancellable as user types.

---

# 74. Incremental Query

Dioxus search box:

```text
i
ir
iro
iroh
```

Previous query tasks should cancel.

---

# 75. Debounce

Example:

```text
100–250 ms
```

before expensive query.

---

# 76. Result Streaming

For large result sets:

```text
first page quickly
then more
```

---

# 77. Pagination

Cursor-based pagination preferable to loading thousands of hits.

---

# 78. Search Cursor

```rust
pub struct SearchCursor {
    // backend-neutral opaque cursor
}
```

---

# 79. Index Update Pipeline

```text
Durable Commit
   ↓
Index Event
   ↓
Index Worker
   ↓
Search Index
```

---

# 80. Persist Before Index

Always:

```text
durable state first
index second
```

---

# 81. Index Lag

Search may briefly lag behind durable state.

UI can tolerate small lag.

---

# 82. Strong Read-Your-Write

For newly sent local message, UI can inject recent unindexed item into results if needed.

Not mandatory initially.

---

# 83. Index Event

```rust
pub enum IndexEvent {
    UpsertMessage(MessageId),
    DeleteMessage(MessageId),
    UpsertFile(BlobId),
    DeleteFile(BlobId),
    UpsertContact(AccountId),
}
```

---

# 84. Index Worker

Bounded background worker.

---

# 85. Batch Indexing

Process events in batches.

Reduces:

```text
disk sync
CPU
lock churn
```

---

# 86. Coalescing

Multiple edits to same document:

```text
index latest state only
```

---

# 87. Index Queue Durability

Two choices:

```text
rebuildable in-memory queue
```

or:

```text
durable index checkpoint
```

Recommended:

```text
durable indexing cursor/checkpoint
```

so crash recovery knows where to continue.

---

# 88. Index Checkpoint

```rust
pub struct IndexCheckpoint {
    pub source_revision: u64,
}
```

---

# 89. Event Log Integration

If Part 04 provides ordered events:

```text
index consumes event sequence
```

---

# 90. Idempotent Indexing

Reprocessing same source event must be safe.

---

# 91. Crash During Index Update

Index backend transaction should prevent partial corruption.

---

# 92. Index Corruption

Detect:

```text
checksum/version/open failure
```

Then:

```text
quarantine/delete index
rebuild
```

Authoritative messages remain safe.

---

# 93. Index Version

```rust
pub struct SearchIndexVersion(pub u32);
```

---

# 94. Schema Upgrade

New analyzer/index schema:

```text
build new index side-by-side
swap when ready
```

---

# 95. Zero-Downtime Reindex

```text
old index serves
new index builds
atomic switch
```

if storage allows.

---

# 96. Low-Storage Reindex

If no space for two copies:

```text
incremental migration
or
disable search temporarily
```

---

# 97. Storage Budget

Index size bounded relative to searchable data.

---

# 98. Index Quota

```rust
pub struct SearchStorageBudget {
    pub max_text_index_bytes: u64,
    pub max_vector_index_bytes: u64,
}
```

---

# 99. Storage Pressure

Under pressure:

```text
drop vector index first
keep basic FTS
```

if semantic search optional.

---

# 100. Emergency Storage Pressure

Part 17 can disable indexing entirely.

---

# 101. Battery-Aware Indexing

Foreground/charging:

```text
full-speed indexing
```

Background/battery saver:

```text
defer expensive semantic embeddings
```

---

# 102. Thermal-Aware Indexing

Pause embedding generation under thermal pressure.

---

# 103. User-Visible Search Still Works

Even if semantic index paused:

```text
basic FTS remains
```

---

# 104. Background Index Scheduler

```rust
pub enum IndexWorkClass {
    Immediate,
    Background,
    Heavy,
}
```

---

# 105. Immediate

Examples:

```text
new message text
message edit
delete
```

---

# 106. Background

Examples:

```text
file metadata
link extraction
```

---

# 107. Heavy

Examples:

```text
PDF text extraction
OCR
embedding generation
```

---

# 108. Heavy Work Conditions

Run when:

```text
charging
unmetered
thermal okay
```

according to user policy.

---

# 109. Link Extraction

Index URLs separately.

---

# 110. Link Search

User can search:

```text
all links in conversation
```

---

# 111. Media Search

Image/video search baseline:

```text
filename
caption
sender
date
```

---

# 112. Image Semantic Search

Future optional:

```text
local vision embedding
```

high cost.

Not initial production requirement.

---

# 113. File-Type Facets

```text
Images
Videos
Documents
Audio
Links
```

---

# 114. Conversation Search Projection

Maintain lightweight metadata for:

```text
latest match
match count
```

if useful.

---

# 115. Search Result Navigation

Hit contains enough logical identity to navigate:

```text
conversation
message
file
```

---

# 116. Jump to Message

Dioxus:

```text
search hit
→ conversation
→ load around MessageId
→ highlight
```

---

# 117. No Offset-Based Identity

Do not store:

```text
"message was row 145"
```

as stable navigation.

Use MessageId.

---

# 118. Deleted Hit Race

If user clicks result after message deleted:

```text
show no longer available
```

not crash.

---

# 119. Permission Changes

If user loses access to group/plugin content:

```text
remove related search entries
```

---

# 120. Device Revocation

Revoked device local data depends on account/device policy.

Search should follow underlying data deletion policy.

---

# 121. Conversation Deletion

If user deletes local conversation:

```text
remove index namespace
```

---

# 122. Archive

Archived conversation remains searchable unless user filters it out.

---

# 123. Hidden/Locked Conversation

Search visibility follows privacy setting.

---

# 124. App Lock

High-security mode may require unlock before search index opens.

---

# 125. Search Index Key

If encrypted index:

```text
derive separate local search key
```

Do not reuse account root key directly.

---

# 126. Key Rotation

Index encryption key can rotate independently.

Because index is rebuildable:

```text
delete/rebuild
```

is acceptable recovery strategy.

---

# 127. Backup

Search index should usually not be backed up.

Rebuild after restore.

---

# 128. Why Not Backup Index

It is:

```text
large
derived
potentially sensitive
version-dependent
```

---

# 129. Restore

After backup restore:

```text
start with empty index
background rebuild
```

---

# 130. Multi-Device Search

Each device maintains its own local index.

Do not synchronize search index files.

---

# 131. Multi-Device Search Consistency

Underlying messages/files sync.

Each device independently indexes what it possesses.

---

# 132. Device-Specific Results

A device may lack old history.

Search naturally returns only locally available data.

---

# 133. Optional Remote Search

If user explicitly wants search on another trusted device:

```text
query remote trusted device
```

could be future feature.

But do not expose plaintext to central server.

---

# 134. Remote Trusted Search

Potential:

```text
device A
→ E2EE query
→ device B
→ local search
→ encrypted results
```

Useful for home node/archive.

---

# 135. Search Query Privacy

Even remote trusted-device search queries are sensitive.

Encrypt them.

---

# 136. No Server Query Logging

If relay forwards encrypted search request:

```text
server sees opaque traffic only
```

---

# 137. Personal Archive Node

Part 20 embedded node can store:

```text
encrypted history
searchable local archive
```

if user explicitly trusts it and gives decryption/search authority.

---

# 138. Archive Security

Grant:

```text
searchable archive capability
```

explicitly.

Not every relay gets it.

---

# 139. Search Capability

```rust
pub enum SearchCapability {
    LocalContent,
    FileContent,
    SemanticSearch,
    RemoteTrustedSearch,
}
```

---

# 140. Plugin Search Provider

Plugin can expose its own search provider.

```rust
pub trait PluginSearchProvider {
    fn search(
        &self,
        query: &PluginSearchQuery,
    ) -> Result<Vec<PluginSearchHit>, PluginError>;
}
```

---

# 141. Federated Local Search

Global UI can combine:

```text
core FTS
plugin providers
semantic index
```

---

# 142. Timeout Per Provider

A slow plugin must not block global search.

---

# 143. Provider Result Budget

Limit:

```text
hits/provider
time/provider
```

---

# 144. Ranking Across Providers

Normalize provider scores before merging.

---

# 145. Plugin Result Labeling

Show source:

```text
Messages
Files
ERP Plugin
```

---

# 146. Plugin Data Privacy

Core UI does not receive more plugin content than plugin is allowed to expose.

---

# 147. Search Suggestions

Suggestions may include:

```text
recent queries
contacts
recent conversations
```

---

# 148. Recent Query Privacy

Stored locally.

Allow clear history.

---

# 149. Search History

Optional.

Not required.

---

# 150. Incognito Search

High-security option:

```text
do not save recent query
```

---

# 151. Search Telemetry

Do not upload raw user queries.

---

# 152. Safe Metrics

Possible aggregate:

```text
query latency
index size
result count
rebuild duration
```

without text.

---

# 153. Diagnostics

Part 18 can show:

```text
index status
documents indexed
index lag
index bytes
rebuild state
semantic model version
```

---

# 154. User Diagnostics

Simple:

```text
Search index is rebuilding
```

---

# 155. Developer Diagnostics

Detailed:

```text
source revision
indexed revision
pending events
FTS size
vector size
last corruption
```

---

# 156. Search Health

```rust
pub enum SearchHealth {
    Ready,
    Indexing,
    Rebuilding,
    Degraded,
    Disabled,
    Failed,
}
```

---

# 157. Degraded

Example:

```text
basic FTS works
semantic index unavailable
```

---

# 158. Search Failure

Search failure must not affect messaging.

---

# 159. Startup

Do not block app startup on full index rebuild.

Flow:

```text
open DB
open index
if healthy → ready
if stale → incremental catch-up
if corrupt → rebuild background
```

---

# 160. Lazy Initialization

Search engine can initialize when:

```text
app idle
or
user opens search
```

depending startup target.

---

# 161. Index Catch-Up

Use source checkpoint.

---

# 162. Message Ingestion

On new message durable commit:

```text
enqueue immediate index update
```

---

# 163. Read Receipt

Does not need full-text reindex.

Only update searchable metadata if search filters depend on read state.

---

# 164. Reactions

Usually no reindex unless reactions searchable.

---

# 165. Attachment Download

Metadata can index before file downloaded.

Content extraction waits until local file exists.

---

# 166. Partial File

Do not parse until enough content/complete file exists.

---

# 167. File Hash Validation

Parse only verified attachment content.

---

# 168. Parser Isolation

Potential parser execution modes:

```text
trusted Rust crate
WASM sandbox
out-of-process parser
```

---

# 169. Parser Resource Limits

Bound:

```text
CPU
RAM
output text size
recursion
file size
```

---

# 170. Zip Bomb

Archive/document parser must protect against compressed bombs.

---

# 171. PDF Bomb

Bound:

```text
page count
object count
text extraction size
```

---

# 172. Malicious Unicode

Normalizer/tokenizer must handle malformed sequences safely.

---

# 173. Fuzzing

Part 10 fuzz:

```text
query parser
tokenizer
index document parser
filter parser
highlighting
search protocol
```

---

# 174. File Parser Fuzzing

Separate corpora per format.

---

# 175. Property Tests

Examples:

```text
deleted message never appears
read-only index corruption never corrupts authoritative DB
replaying index event is idempotent
search result IDs always resolve or fail safely
```

---

# 176. Rebuild Test

Delete index completely.

Rebuild.

Results match expected corpus.

---

# 177. Crash During Reindex

Restart.

Continue/restart safely.

---

# 178. Model Upgrade Test

Embedding model v1 → v2.

No mixed vector comparisons.

---

# 179. Low-Storage Test

Index reaches quota.

Semantic indexing pauses before basic search breaks.

---

# 180. Battery Saver Test

Heavy embedding/parser work deferred.

---

# 181. Search While Rebuilding

Return partial/current results with:

```text
Indexing…
```

status.

---

# 182. Multi-Language Test

Search corpus with:

```text
English
Hindi
Urdu
Arabic
mixed code
```

---

# 183. Emoji Search

Search by emoji where tokenizer supports.

---

# 184. File Search Test

Filename, caption, sender, date filters.

---

# 185. Conversation Scope Test

Global vs one conversation results.

---

# 186. Privacy Test

Server receives no plaintext query or index content.

---

# 187. Plugin Isolation Test

Plugin cannot inspect core search index.

---

# 188. Search Performance

Targets:

```text
common query <100 ms on moderate local history
```

where device/storage allows.

Use measurement, not rigid guarantee.

---

# 189. Large History Benchmark

Test:

```text
10k messages
100k messages
1M messages
```

depending product scale.

---

# 190. Index Build Benchmark

Measure:

```text
docs/sec
bytes/sec
CPU
battery
```

---

# 191. Memory Budget

Query should not load entire result corpus into RAM.

---

# 192. Snippet Budget

Limit snippet length.

---

# 193. Result Limit

Default:

```text
20–50
```

per page.

---

# 194. Cancellation Benchmark

Rapid query typing should not leave dozens of expensive tasks alive.

---

# 195. Concurrent Search

Limit active searches.

---

# 196. Dioxus Search UI

Components:

```text
SearchBar
FilterChips
ResultList
ResultSection
SearchStatus
```

---

# 197. Global Search UX

Sections:

```text
Messages
Files
Contacts
Links
Plugins
```

---

# 198. Conversation Search UX

Show:

```text
match count
previous/next match
```

---

# 199. Jump Navigation

Use MessageId and lazy history loading.

---

# 200. Search Highlight

Highlight matched terms.

---

# 201. Semantic Match Label

If semantic result not exact keyword match:

```text
Related result
```

optional.

---

# 202. Explainability

For baseline ranking, provide simple reason:

```text
matched message text
matched filename
recent
```

---

# 203. No AI Hallucination

Search returns real stored documents/messages only.

Semantic search ranks existing content; it does not invent answers.

---

# 204. Local Knowledge Retrieval

Future assistant-like feature can retrieve:

```text
top local messages/files
```

then answer locally or through user-approved model.

---

# 205. Retrieval API

```rust
pub trait KnowledgeRetriever {
    fn retrieve(
        &self,
        request: RetrievalRequest,
    ) -> Result<Vec<KnowledgeChunk>, SearchError>;
}
```

---

# 206. Knowledge Chunk

```rust
pub struct KnowledgeChunk {
    pub source: SearchDocumentId,
    pub text: String,
    pub score: f32,
}
```

---

# 207. Retrieval Privacy

Do not send retrieved private chunks to external AI API without explicit user configuration/consent.

---

# 208. On-Device AI

If local model available:

```text
retrieve locally
→ infer locally
```

preserves privacy.

---

# 209. External AI Adapter

If user configures API key:

```text
clear data-flow warning
scope selected content
```

Do not silently upload conversation history.

---

# 210. Retrieval Scope

Assistant can be restricted:

```text
this conversation only
selected files
selected workspace
```

---

# 211. Plugin Knowledge Providers

Plugins may contribute retrievable chunks from own namespace.

---

# 212. Search Security Boundary

Search engine sees plaintext after local decryption.

Therefore it belongs inside trusted local runtime, not untrusted plugin.

---

# 213. Process Isolation

Optional high-security architecture:

```text
search/index worker process
```

with local authenticated IPC.

Not required initially.

---

# 214. Headless Search

Part 16 daemon can expose local search API to:

```text
CLI
desktop UI
mobile client
```

---

# 215. Local IPC Search

Use typed request/response.

Bound query/result sizes.

---

# 216. Remote Admin Search

Do not expose user-content search over remote admin API by default.

---

# 217. Embedded Archive Search

Optional trusted personal node can serve E2EE-authenticated search to user's devices.

---

# 218. FFI Search

Part 19 exposes:

```text
search
cancel_search
fetch_more
```

with opaque handles.

---

# 219. WASM Components

Do not run entire core index in WASM by default.

WASM may implement:

```text
custom token transforms
plugin search
ranking policy
```

if bounded.

---

# 220. Search Protocol

If remote trusted-device search added:

```text
search/1
```

with:

```text
encrypted query
scope
limit
result metadata
```

---

# 221. Search Query Abuse

Remote trusted device still gets quotas.

---

# 222. Result Redaction

Remote archive returns only authorized scope.

---

# 223. Search Capability Negotiation

Part 07 can advertise:

```text
local-search-v1
remote-trusted-search-v1
semantic-search-v1
```

---

# 224. Interoperability

Part 23 can define:

```text
query parser vectors
remote search request/response vectors
```

for externally implemented clients.

---

# 225. Search Index Version Compatibility

Index files are local implementation detail.

Do not standardize them across implementations.

---

# 226. Standardize Search API Semantics

Cross-language SDK should agree on:

```text
scope
filters
pagination
hit identity
```

not backend index format.

---

# 227. Suggested Workspace

```text
crates/
├── comm-search-core/
├── comm-search-index/
├── comm-search-fts/
├── comm-search-files/
├── comm-search-semantic/
├── comm-search-ranking/
├── comm-search-runtime/
├── comm-search-knowledge/
├── comm-search-diagnostics/
└── comm-search-testkit/
```

---

# 228. `comm-search-core`

Owns:

```text
query
scope
filters
hits
document IDs
```

---

# 229. `comm-search-index`

Backend-neutral index trait and lifecycle.

---

# 230. `comm-search-fts`

Owns text indexing/tokenization.

---

# 231. `comm-search-files`

Owns attachment metadata/content extraction pipeline.

---

# 232. `comm-search-semantic`

Feature-gated:

```text
embedding
vector index
model version
```

---

# 233. `comm-search-ranking`

Owns ranking/fusion.

---

# 234. `comm-search-runtime`

Owns:

```text
index worker
checkpoints
rebuild
resource policy
```

---

# 235. `comm-search-knowledge`

Owns retrieval API for local assistant/knowledge use.

---

# 236. `comm-search-testkit`

Provides:

```text
synthetic histories
multilingual corpus
corrupt index fixtures
fake parser
fake embedding model
```

---

# 237. Implementation Phases

## Phase 1 — Core Message FTS

```text
message body
conversation scope
sender/date filters
```

## Phase 2 — Contacts / Files / Links

```text
metadata indexing
facets
navigation
```

## Phase 3 — Incremental Indexing

```text
checkpoint
crash recovery
rebuild
```

## Phase 4 — Privacy Hardening

```text
index encryption option
backup exclusion
app-lock behavior
```

## Phase 5 — File Content

```text
safe parsers
resource limits
PDF/text/Markdown
```

## Phase 6 — Semantic Search

```text
local embeddings
vector index
hybrid ranking
```

## Phase 7 — Knowledge Retrieval

```text
retrieval API
on-device assistant integration
```

## Phase 8 — Remote Trusted Search

```text
personal archive node
E2EE query/result
```

## Phase 9 — Hardening

```text
large corpus
low storage
battery
multilingual
corruption
fuzz
```

---

# 238. Initial Production Recommendation

For v1:

```text
local full-text message search
conversation-scoped search
contacts
filenames
links
date/sender/file filters
incremental rebuildable index
```

Then add:

```text
document content extraction
semantic/vector search
local knowledge retrieval
trusted archive-node search
```

Do not make semantic search a prerequisite for basic search.

---

# 239. Definition of Done

Part 32 is complete when:

- search works fully offline
- the search index is derived/rebuildable, never authoritative
- message edits update one stable search document
- deleted/expired messages are removed from results
- files, contacts, groups, and links can be searched
- query scopes and filters are typed and bounded
- indexing happens only after durable source commit
- indexing is incremental and crash-safe
- corrupt index can be discarded/rebuilt without data loss
- search does not block application startup during rebuild
- background indexing obeys battery/thermal/storage budgets
- file-content parsers are bounded and isolated
- plaintext message/index data is not uploaded to server by default
- semantic embeddings are treated as sensitive data
- semantic search remains optional/local-first
- each device maintains its own local index
- backup restore rebuilds the index rather than trusting stale index files
- plugins can index/search only their authorized namespace
- global search can federate multiple local providers without a slow provider blocking the UI
- search results use stable logical IDs for navigation
- private local knowledge retrieval does not silently send content to external AI
- fuzz, corruption, low-storage, multilingual, crash/rebuild, and large-history tests exist

---

# 240. Relationship to Earlier Parts

Part 32 integrates with:

```text
04 — Offline Event Log
05 — File / Blob Subsystem
08 — Resource Limits
09 — Crash Recovery
10 — Fuzzing
13 — Battery Scheduling
16 — Daemon / Headless Runtime
18 — Diagnostics
19 — C ABI / FFI
20 — Embedded Linux
21 — Protocol Extensions
22 — WASM Components
23 — Interoperability
24 — Plugin Ecosystem
28 — Security / E2EE
30 — Presence / Read State
31 — Background Delivery
```

---

# 241. Final Architecture

```text
               AUTHORITATIVE LOCAL STATE
                         │
        ┌────────────────┼────────────────┐
        │                │                │
     Messages          Files           Contacts
        │                │                │
        └────────────────┼────────────────┘
                         │
                   Index Event Stream
                         │
                  Background Indexer
             ┌───────────┴────────────┐
             │                        │
         Text / FTS            Semantic Vectors
             │                        │
             └───────────┬────────────┘
                         │
                    Query Engine
                         │
                    Ranking/Fusion
                         │
                      Dioxus
```

Privacy boundary:

```text
E2EE ciphertext at rest/server
        │
local trusted device decrypts
        │
local search index
        │
local query/result
```

No default path exists for:

```text
plaintext search index
→ central server
```

---

# 242. Final Principle

Search should make the user's local history useful without turning encrypted communication into a searchable server-side data lake.

The correct model is:

```text
authoritative encrypted/durable data
+
local derived index
+
bounded incremental indexing
+
optional local semantic retrieval
+
rebuildable state
```

not:

```text
upload plaintext/tokenized history
so the server can search it
```

Part 32 therefore gives the platform fast, modern discovery while preserving the local-first and E2EE guarantees established by the earlier architecture.
