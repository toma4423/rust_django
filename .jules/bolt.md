
## 2025-05-14 - [Optimize has_perm with a single join query]
**Learning:** Permission checks involving direct and group-based permissions can be consolidated into a single query using left joins and an OR condition (`Condition::any()`). This significantly reduces database round-trips from up to 3 to exactly 1.
**Action:** Always look for opportunities to consolidate multiple sequential SeaORM queries that use intermediate IDs (like `group_ids`) into a single joined query.
