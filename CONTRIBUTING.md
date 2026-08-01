# Contributing Guidelines

Thank you for considering contributing to `rtsc`! To maintain codebase quality and a clean git history, please follow these guidelines when submitting contributions.

---

## 1. Commit Message Convention (Mandatory)

All commit messages **must** follow the [Conventional Commits](https://www.conventionalcommits.org/) specification format:

```text
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

### 1.1 Commit Types

The `<type>` must be one of the following:

* `feat`: A new feature
* `fix`: A bug fix
* `docs`: Documentation only changes
* `style`: Code style/formatting (white-space, formatting, missing semi-colons, etc.)
* `refactor`: A code change that neither fixes a bug nor adds a feature
* `perf`: A code change that improves performance
* `test`: Adding missing tests or correcting existing tests
* `chore`: Maintenance tasks, dependencies, tooling, or build configuration
* `build`: Changes that affect the build system or external dependencies
* `ci`: Changes to CI configuration scripts and workflows

### 1.2 Subject Line Guidelines

* Use **imperative mood** and present tense ("add", not "added" or "adds").
* Keep the subject line concise (aim for under 50 characters, maximum 72 characters).
* Capitalizing the first letter is optional.
* **Do not** end the subject line with a period (`.`).

### 1.3 Example Commit Messages

```text
feat(parser): add support for async functions

fix(lexer): resolve unexpected token error on EOF

docs(readme): update build and setup instructions
```

---

## 2. Developer Certificate of Origin (DCO) Sign-Off (Mandatory)

All commits submitted to this project **must include a DCO sign-off**. By adding a sign-off, you certify that you have the right to submit the code under the project's license.

### 2.1 How to Sign Off

Include a `Signed-off-by:` line at the end of your commit message:

```text
Signed-off-by: Jane Doe <jane.doe@example.com>
```

You can automatically add this line when committing using the `-s` or `--signoff` flag with Git:

```bash
git commit -s -m "feat(scope): add new feature"
```

> **Note:** Commits without a valid DCO sign-off will fail automated checks and cannot be merged.

---

## 3. Pull Request Process

1. Fork or branch off of `main`.
2. Create a focused feature branch for your changes (e.g., `feature/my-feature` or `fix/my-fix`).
3. Ensure all tests pass and code compiles cleanly.
4. Keep commits small, logically structured, conventional, and signed off with DCO (`git commit -s`).
5. Open a Pull Request detailing your changes and referencing relevant issue numbers.
