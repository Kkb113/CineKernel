# Scene graph contract

Scene is the authoritative renderer-facing representation: composition metadata plus a finite NodeKind hierarchy, optional numeric NodeId, visual fields, media references, layout, effects, and selected 3D placement. Runtime pixels are not serialized.

The architecture graph records 18 boundaries into or out of this model. Numeric IDs support renderer/timeline targeting, but do not establish a source map back to Cinema entries, React components, user instructions, or agent operations.
