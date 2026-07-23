CREATE INDEX IF NOT EXISTS idx_actors_embedding
ON actors USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS idx_messages_embedding
ON messages USING hnsw (embedding vector_cosine_ops);
