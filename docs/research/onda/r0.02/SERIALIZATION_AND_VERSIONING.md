# Serialization and versioning

Scene serialization uses **JSON** with current version **1**. Current-version omission is `true` and runtime pixels are not serialized. TypeScript and Rust meet at JSON boundaries, but forward-field, future-version, and semantic migration guarantees remain incomplete and require focused compatibility fixtures in later research.
