## 2024-05-18 - Added Accessibility Labels to Checkboxes and Search
**Learning:** The existing admin templates lacked `aria-label`s for the action checkboxes (select all / select item) and the search inputs, and had decorative SVGs that were visible to screen readers.
**Action:** When adding or modifying interactive UI components like inputs, buttons, and checkboxes, ensure they are accessible by adding appropriate `aria-label`s, `title`s, or hiding decorative items with `aria-hidden="true"`.

## 2024-05-30 - Added Required Field Indicator in Forms
**Learning:** The admin form templates already included logic to append a `.required` class to labels of required fields, but this class had no associated styling in the `style.css` file. Users and screen readers could miss that these fields are mandatory, breaking a common UX pattern.
**Action:** Always verify that dynamically added accessibility or state classes (like `.required`, `.disabled`, etc.) have corresponding CSS rules defined in the styling system, and apply standard conventions (e.g., a red asterisk for required fields).
