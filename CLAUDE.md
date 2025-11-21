- Never use icons in text display to users.
- Always prefer simpler, more readable code rather than complexity.
- Be vigilant about identifying and refactoring dead code.
- Prefer short commit messages.

## Recent Work

Added jar file reading using the `zip` crate - can scan a mods folder for jars, then view contents filtered to `data/*/recipe/` or `data/*/recipes/` paths for recipe extraction. Next step is to extract and parse the actual recipe JSON files.