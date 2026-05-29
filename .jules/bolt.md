## 2024-05-29 - Resolve N+1 issues in SeaORM with load_many
**Learning:** In SeaORM, iterating over a collection of records and calling `find_related(...).all(...)` on each one causes an N+1 query performance bottleneck. This pattern occurs in `src/controllers/admin.rs` for fetching user groups.
**Action:** Always batch related record queries using SeaORM's `load_many(RelatedEntity, db)` on the collection of models, and then zip or map the results together instead of running sequential queries inside a loop.
