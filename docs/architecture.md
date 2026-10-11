# Architecture

## General prospectus questions
1. The Rust server receives the natural-language question.
2. `RagIndex` searches indexed prospectus chunks using the existing BM25-style ranker.
3. The top retrieved chunks, including source/section/page metadata, are passed to `src/generator.rs`.
4. `generator.rs` invokes the local Ollama model (default `llama3.2:3b`) and constrains it to answer from retrieved context.
5. `rag.rs` appends citations derived from the retrieved chunks. Page numbers are shown only when present in the page-marked source dataset.

## Preserved original features
- APS calculation remains handled by `aps.rs`.
- Marks/results detection and course prediction remain handled by the original APS functions.
- Existing structured programme matching remains in the original route.
- Existing course-information matching remains in place; free-text prospectus answers use retrieval plus generation.

## Limitations
The retrieval system is lexical BM25-style retrieval, not a vector embedding index. The generation stage is still RAG because retrieved document passages are supplied to a language model to generate the answer. Citation correctness depends on retrieval relevance and source metadata.
