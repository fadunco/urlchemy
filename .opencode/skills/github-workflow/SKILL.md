---
name: github-workflow
description: Prepare commits, branches, GitHub repositories or pull requests using git and gh, with account checks, focused diffs and approval before publication.
---

# GitHub workflow

1. Read AGENTS.md and inspect the working tree and remotes. Preserve unrelated changes. If the directory is not a git repository, ask before initializing one unless initialization was explicitly requested.
2. Check `gh auth status`. If login is needed, ask the user to run `gh auth login --web` in their own terminal. Never retrieve tokens, read credential stores, or paste credentials into chat/configuration.
3. Confirm the authenticated GitHub login (for example, `gh api user --jq .login`) and the intended owner, repository name, visibility and license before repository creation. Do not infer ownership from the filesystem username.
4. Work in a focused branch when implementing an approved task. Run documented verification. Review `git diff` and explicitly stage intended files only. Commit only when requested, with an accurate message.
5. Before any remote mutation, show the proposed target and effect and obtain approval. This includes repo creation, push, PR/issue creation or editing, releases and merge. Approval for a local commit is not permission to push.
6. A PR should describe behavior, verification actually performed, known limitations, and relevant screenshots for UI changes. Do not invent check results or post private configuration/transcripts.
7. Do not force-push, reset, discard changes, change repository visibility, merge or delete branches without explicit authorization. Do not install a GitHub MCP for actions `gh` already supports.

Use read-only inspection where possible. Ordinary gh read commands still require the permissions configured by the project. Stop on authentication or authorization errors instead of attempting alternate credentials.
