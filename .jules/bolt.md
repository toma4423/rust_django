## 2024-05-29 - Resolve N+1 issues in SeaORM with load_many
**Learning:** In SeaORM, iterating over a collection of records and calling `find_related(...).all(...)` on each one causes an N+1 query performance bottleneck. This pattern occurs in `src/controllers/admin.rs` for fetching user groups.
**Action:** Always batch related record queries using SeaORM's `load_many(RelatedEntity, db)` on the collection of models, and then zip or map the results together instead of running sequential queries inside a loop.

## 2026-05-30 - Optimize user permission check query
**Learning:** Checking permissions by sequentially querying direct permissions and then group permissions causes multiple DB round-trips. Consolidating these into a single query using LEFT JOINs and an OR condition () is more efficient.
**Action:** Use joins and complex conditions to reduce the number of queries for existence checks across multiple relationship paths.

## 2025-05-30 - Optimize user permission check query
**Learning:** Checking permissions by sequentially querying direct permissions and then group permissions causes multiple DB round-trips. Consolidating these into a single query using LEFT JOINs and an OR condition (`Condition::any()`) is more efficient.
**Action:** Use joins and complex conditions to reduce the number of queries for existence checks across multiple relationship paths.

## 2025-05-30 - Prevent unnecessary updates with is_changed()
**Learning:** SeaORM executes an `UPDATE` query even if the model fields haven't actually changed when calling `.update()`. If we assign new values to `ActiveModel` fields without checking if they are actually different from the `existing` values, `.is_changed()` will be true and the redundant update will occur.
**Action:** When updating a model, always check if the new value differs from the existing value before assigning it to the `ActiveModel` field using `Set(new_value)`. Then, wrap the `update()` call in `if active_model.is_changed() { ... }` to avoid round-trips to the database when no modifications occurred.
## 2026-06-01 - Optimize toggle_todo redundant query
**Learning:** When toggling or updating a record that requires returning associated related data for UI rendering, developers sometimes fetch the item, update it, and then re-fetch it using a JOIN. This is a common anti-pattern that leads to redundant database round-trips.
**Action:** Use SeaORM's `.find_also_related()` in the initial query. The related data is returned as a tuple alongside the original model. You can convert the model into an `ActiveModel` for the update, execute the update, and then safely return the newly updated model along with the preserved, already-fetched related data without a second query.
