---
description: Review recent code changes for clarity and simplicity
---

Review all recent uncommitted changes in this repository. Use `git diff` and `git status` to identify what has been modified.

For each change, evaluate against these priorities:

1. **CLAUDE.md compliance** - Verify changes follow all project and user instructions
2. **Simplicity over complexity** - Flag any over-engineered solutions; suggest simpler alternatives
3. **Readability** - Code should be immediately understandable to a human reader
4. **Dead code** - Identify any unused variables, functions, or imports that should be removed
5. **Easiest approach** - Question whether there's a more straightforward way to achieve the same result

For each issue found, provide:
- File and line number
- What the problem is
- A concrete suggestion for improvement

Be direct and specific. Skip praise - focus only on actionable feedback.
