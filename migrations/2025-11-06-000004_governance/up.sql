-- governance tracking for protocol parameter changes affecting trading

CREATE TABLE IF NOT EXISTS governance_proposals (
    proposal_id BIGINT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    outcome TEXT,
    deposit_amount NUMERIC(39, 0),
    start_block_height BIGINT,
    end_block_height BIGINT,
    start_timestamp TIMESTAMPTZ,
    end_timestamp TIMESTAMPTZ,
    quorum NUMERIC(10, 4),
    payload JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_governance_proposals_state ON governance_proposals(state);
CREATE INDEX idx_governance_proposals_kind ON governance_proposals(kind);
CREATE INDEX idx_governance_proposals_start ON governance_proposals(start_block_height);
CREATE INDEX idx_governance_proposals_end ON governance_proposals(end_block_height);

CREATE TABLE IF NOT EXISTS governance_votes (
    id BIGSERIAL PRIMARY KEY,
    proposal_id BIGINT NOT NULL,
    validator_identity_key TEXT,
    vote TEXT NOT NULL,
    voting_power NUMERIC(39, 0),
    effective_voting_power NUMERIC(39, 0),
    voting_power_percentage NUMERIC(10, 4),
    parent_validator_identity_key TEXT,
    block_height BIGINT NOT NULL,
    voted_at TIMESTAMPTZ NOT NULL,
    tx_hash BYTEA,
    FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
);

CREATE INDEX idx_governance_votes_proposal ON governance_votes(proposal_id);
CREATE INDEX idx_governance_votes_validator ON governance_votes(validator_identity_key);
CREATE INDEX idx_governance_votes_vote ON governance_votes(vote);
CREATE INDEX idx_governance_votes_height ON governance_votes(block_height);

CREATE TABLE IF NOT EXISTS governance_parameters (
    chain_id TEXT PRIMARY KEY,
    deposit_amount NUMERIC(39, 0),
    passing_threshold NUMERIC(10, 2),
    slashing_threshold NUMERIC(10, 2),
    valid_quorum NUMERIC(10, 2),
    proposal_voting_blocks BIGINT,
    updated_height BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    raw_params JSONB
);

CREATE INDEX idx_governance_parameters_updated ON governance_parameters(updated_height);

-- view: active proposals
CREATE VIEW active_governance_proposals AS
SELECT
    proposal_id,
    title,
    kind,
    state,
    start_block_height,
    end_block_height,
    start_timestamp,
    end_timestamp,
    quorum
FROM governance_proposals
WHERE state IN ('voting', 'submitted')
ORDER BY end_block_height ASC;

-- view: proposal voting summary
CREATE VIEW proposal_voting_summary AS
SELECT
    p.proposal_id,
    p.title,
    p.state,
    COUNT(v.id) as total_votes,
    SUM(CASE WHEN v.vote = 'yes' THEN v.effective_voting_power ELSE 0 END) as yes_power,
    SUM(CASE WHEN v.vote = 'no' THEN v.effective_voting_power ELSE 0 END) as no_power,
    SUM(CASE WHEN v.vote = 'abstain' THEN v.effective_voting_power ELSE 0 END) as abstain_power,
    SUM(v.effective_voting_power) as total_voting_power
FROM governance_proposals p
LEFT JOIN governance_votes v ON p.proposal_id = v.proposal_id
GROUP BY p.proposal_id, p.title, p.state;
