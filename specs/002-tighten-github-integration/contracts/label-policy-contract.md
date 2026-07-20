# Contract: Label Policy (Lifecycle + Relationships)

## Purpose

Define canonical label semantics independent of provider.

## Canonical Label Groups

## Lifecycle labels
- `lifecycle:todo`
- `lifecycle:in-progress`
- `lifecycle:blocked`
- `lifecycle:done`

## Relationship labels
- `rel:parent`
- `rel:child`
- `rel:blocked-by`
- `rel:depends-on`

## Contract Rules

1. Lifecycle metadata state MUST map to exactly one lifecycle label where supported.
2. Relationship metadata edges MUST map to corresponding relationship labels where supported.
3. Provider adapters MUST provide a mapping table from canonical labels to provider-native labels.
4. If a provider does not support labels, adapter MUST emit explicit unsupported capability and skip silent no-op behavior.
5. Reconciliation MUST restore missing required labels or emit `conflict_flagged` with reason.
6. `apply_sync` MUST reject unknown canonical labels before provider-specific mapping.

## Validation

- Label-to-metadata and metadata-to-label checks run in conformance suite.
- Unknown labels cannot overwrite canonical mappings.
