## 2024-05-29 - Resolve N+1 issues in SeaORM with load_many
**Learning:** In SeaORM, iterating over a collection of records and calling `find_related(...).all(...)` on each one causes an N+1 query performance bottleneck. This pattern occurs in `src/controllers/admin.rs` for fetching user groups.
**Action:** Always batch related record queries using SeaORM's `load_many(RelatedEntity, db)` on the collection of models, and then zip or map the results together instead of running sequential queries inside a loop.

## 2026-05-30 - Optimize user permission check query
**Learning:** Checking permissions by sequentially querying direct permissions and then group permissions causes multiple DB round-trips. Consolidating these into a single query using LEFT JOINs and an OR condition () is more efficient.
**Action:** Use joins and complex conditions to reduce the number of queries for existence checks across multiple relationship paths.

## 2025-05-30 - Optimize user permission check query
**Learning:** Checking permissions by sequentially querying direct permissions and then group permissions causes multiple DB round-trips. Consolidating these into a single query using LEFT JOINs and an OR condition (`Condition::any()`) is more efficient.
**Action:** Use joins and complex conditions to reduce the number of queries for existence checks across multiple relationship paths.
