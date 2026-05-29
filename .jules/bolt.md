## 2024-05-29 - Resolve N+1 issues in SeaORM with load_many
**Learning:** In SeaORM, iterating over a collection of records and calling `find_related(...).all(...)` on each one causes an N+1 query performance bottleneck. This pattern occurs in `src/controllers/admin.rs` for fetching user groups.
**Action:** Always batch related record queries using SeaORM's `load_many(RelatedEntity, db)` on the collection of models, and then zip or map the results together instead of running sequential queries inside a loop.

## 2025-05-14 - [Optimize has_perm with a single join query]
**Learning:** Permission checks involving direct and group-based permissions can be consolidated into a single query using left joins and an OR condition (`Condition::any()`). This significantly reduces database round-trips from up to 3 to exactly 1.
**Action:** Always look for opportunities to consolidate multiple sequential SeaORM queries that use intermediate IDs (like `group_ids`) into a single joined query.
