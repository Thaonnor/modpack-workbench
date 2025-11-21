- Never use icons in text display to users.
- Always prefer simpler, more readable code rather than complexity.
- Be vigilant about identifying and refactoring dead code.
- Prefer short commit messages.

## Recent Work

Recipe extraction and display implemented. Recipes are stored in SQLite with full JSON preserved. Progress bar now shows recipe count with batched updates (every 50 recipes). Shaped crafting recipes display in a visual 3x3 grid with normalized keys (A, B, C, etc.) mapped to ingredients. Non-shaped recipes fall back to comma-separated ingredient list.