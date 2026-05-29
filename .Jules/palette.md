## 2024-05-18 - Added Accessibility Labels to Checkboxes and Search
**Learning:** The existing admin templates lacked `aria-label`s for the action checkboxes (select all / select item) and the search inputs, and had decorative SVGs that were visible to screen readers.
**Action:** When adding or modifying interactive UI components like inputs, buttons, and checkboxes, ensure they are accessible by adding appropriate `aria-label`s, `title`s, or hiding decorative items with `aria-hidden="true"`.
