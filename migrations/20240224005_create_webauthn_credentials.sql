-- Create the WebAuthn credentials table
CREATE TABLE webauthn_credentials (
  uuid UUID PRIMARY KEY,
  credential_id TEXT NOT NULL,
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  name VARCHAR(255) NOT NULL,
  aaguid BYTEA NOT NULL,
  public_key BYTEA NOT NULL,
  counter INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_used_at TIMESTAMPTZ
);

-- Create indexes
CREATE UNIQUE INDEX webauthn_credentials_credential_id_idx ON webauthn_credentials (credential_id);
CREATE INDEX webauthn_credentials_user_id_idx ON webauthn_credentials (user_id);
CREATE INDEX webauthn_credentials_tenant_id_idx ON webauthn_credentials (tenant_id);

-- Add foreign key constraints
ALTER TABLE webauthn_credentials
  ADD CONSTRAINT fk_webauthn_credentials_user_id
  FOREIGN KEY (user_id) REFERENCES users(id)
  ON DELETE CASCADE;

ALTER TABLE webauthn_credentials
  ADD CONSTRAINT fk_webauthn_credentials_tenant_id
  FOREIGN KEY (tenant_id) REFERENCES tenants(id)
  ON DELETE CASCADE;

-- Add tenant isolation policy
ALTER TABLE webauthn_credentials ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_policy ON webauthn_credentials
  USING (tenant_id::text = current_setting('app.tenant_id', TRUE));

-- Grant permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON webauthn_credentials TO app_user;