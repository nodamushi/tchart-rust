---
name: doc-spec
description: Format and structure for docs/spec.md. Use when creating or updating the specification document.
---

# Prohibited Actions

When designing, you must never read the source code.
The code should depend on the design; the design must not depend on the code.

Unless explicitly permitted, you must not access any directories other than the `docs` directory.

# `docs/spec.md` Format

## Root file structure

Write the following sections in `docs/spec.md` (the root file):

1. **Purpose and Background** — Why this project exists and its context.
2. **Overview** — High-level summary of what it does.
3. **Usage** — How to use it. Include the following subsections as applicable:
   - CLI usage
   - How to run tests
   - How to embed in a web context
   - Each subsection has a brief summary in the root file. If details are long, move them to `docs/spec/<topic>.md` and link from the root file.
4. **Project Structure Overview** — Brief description of the directory/module layout.
5. **Implementation Design Details** — Interfaces, algorithms, and other design definitions (not the implementation itself).
   - Write a rough outline in the root file first.
   - Then create detailed files under `docs/spec/` and link to them.
   - Keep `docs/spec.md` concise: links and summaries only, to minimize reading burden for humans and AI.
6. **Links to External Resources** — References, related docs, etc.

## Approach

The specifications form the foundation of everything. Rather than making assumptions or decisions on your own, determine key points and major design principles through dialogue with the user.

Always seek final approval from the user at the end. When doing so, clearly explain what has changed and how.

If you wait until everything is built to ask for the user’s approval, they will be unable to process the sheer volume of information. Seek detailed confirmation from the user at the stages when establishing the broad overall direction and the specific details of each approach.
