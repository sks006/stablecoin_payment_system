CREATE TYPE payment_status AS ENUM ('pending', 'submitted', 'confirmed', 'failed');

CREATE TABLE payments (
    id UUID PRIMARY KEY,
    idempotency_key VARCHAR(255) UNIQUE NOT NULL,
    amount BIGINT NOT NULL,
    sender VARCHAR(88) NOT NULL,
    recipient VARCHAR(88) NOT NULL,
    status payment_status NOT NULL DEFAULT 'pending',
    signature VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE idempotency_keys (
    key VARCHAR(255) PRIMARY KEY,
    response_body JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
