# 📝 Commit Message Policy

This document defines commit message conventions following the [Conventional Commits](https://www.conventionalcommits.org/) specification with emoji prefixes.

## 🎯 Basic Format

```
<emoji> <type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Examples

```
✨ feat: add user authentication feature
🐛 fix(auth): resolve session management bug during login
📚 docs: update API documentation
💥 feat(api)!: send email to customer when product is shipped
```

## 🎨 Commit Types with Emojis

### Primary Types (Conventional Commits Standard)

| Emoji | GitHub Code               | Type       | Description                                               | Example                                        |
| ----- | ------------------------- | ---------- | --------------------------------------------------------- | ---------------------------------------------- |
| ✨    | `:sparkles:`              | `feat`     | A new feature                                             | `✨ feat: add dark mode toggle`                |
| 🐛    | `:bug:`                   | `fix`      | A bug fix                                                 | `🐛 fix: resolve null pointer exception`       |
| 📚    | `:books:`                 | `docs`     | Documentation only changes                                | `📚 docs: update installation guide`           |
| 🎨    | `:art:`                   | `style`    | Changes that don't affect code meaning (formatting, etc.) | `🎨 style: format code with prettier`          |
| ♻️    | `:recycle:`               | `refactor` | Code change that neither fixes a bug nor adds a feature   | `♻️ refactor: extract auth logic into service` |
| ⚡    | `:zap:`                   | `perf`     | Performance improvements                                  | `⚡ perf: lazy load images`                    |
| ✅    | `:white_check_mark:`      | `test`     | Adding or correcting tests                                | `✅ test: add unit tests for user service`     |
| 🏗️    | `:building_construction:` | `build`    | Changes to build system or dependencies                   | `🏗️ build: update webpack to v5`               |
| 👷    | `:construction_worker:`   | `ci`       | Changes to CI configuration files and scripts             | `👷 ci: add GitHub Actions workflow`           |
| 🧹    | `:broom:`                 | `chore`    | Other changes that don't modify src or test files         | `🧹 chore: update .gitignore`                  |
| ⏪    | `:rewind:`                | `revert`   | Reverts a previous commit                                 | `⏪ revert: revert commit abc123`              |

### Additional Types (Extended)

| Emoji | GitHub Code              | Type         | Description                     | Example                                |
| ----- | ------------------------ | ------------ | ------------------------------- | -------------------------------------- |
| 🔒    | `:lock:`                 | `security`   | Security improvements or fixes  | `🔒 security: patch XSS vulnerability` |
| ⬆️    | `:arrow_up:`             | `deps`       | Dependency updates              | `⬆️ deps: upgrade React to v18`        |
| 🔧    | `:wrench:`               | `config`     | Configuration changes           | `🔧 config: update ESLint rules`       |
| 🏷️    | `:label:`                | `release`    | Release/version related changes | `🏷️ release: bump version to 1.2.0`    |
| 🚑    | `:ambulance:`            | `hotfix`     | Critical hotfix                 | `🚑 hotfix: fix production crash`      |
| 💄    | `:lipstick:`             | `ui`         | UI/UX improvements              | `💄 ui: improve button styling`        |
| 🌐    | `:globe_with_meridians:` | `i18n`       | Internationalization            | `🌐 i18n: add Japanese translations`   |
| ♿    | `:wheelchair:`           | `a11y`       | Accessibility improvements      | `♿ a11y: add screen reader support`   |
| 📸    | `:camera_flash:`         | `snapshot`   | Update snapshots                | `📸 snapshot: update test snapshots`   |
| 🧪    | `:test_tube:`            | `experiment` | Experimental changes            | `🧪 experiment: try new algorithm`     |

## 🔍 Scope

Scope provides additional contextual information and should be a noun describing a section of the codebase.

### Common Scopes

- `api` - API related changes
- `auth` - Authentication/authorization
- `core` - Core functionality
- `db` - Database related
- `deps` - Dependencies
- `ui` - User interface
- `config` - Configuration
- `build` - Build system
- `test` - Testing
- `docs` - Documentation
- `ci` - Continuous integration

### Examples with Scope

```
✨ feat(auth): implement OAuth2 login
🐛 fix(api): handle empty response correctly
📚 docs(readme): add troubleshooting section
⚡ perf(db): optimize query performance
```

## 💥 Breaking Changes

Breaking changes MUST be indicated by:

1. **`!` after type/scope**: `💥 feat(api)!: change response format`
2. **BREAKING CHANGE in footer**:

```
✨ feat: allow provided config object to extend other configs

BREAKING CHANGE: `extends` key in config file is now used for extending other config files
```

## 📏 Rules

### Before Committing

**Always ensure that tests, linting, and builds pass before committing:**

- Run all tests and verify they pass
- Run linting tools and fix any issues
- Ensure the project builds successfully
- Never commit broken code

### Requirements

1. **Emoji is mandatory**: Every commit must start with an appropriate emoji
2. **Type is mandatory**: Every commit must have a type after the emoji
3. **Description is mandatory**: Must be in present tense, not capitalized, no period at the end
4. **Line length**:
   - Subject line: max 72 characters
   - Body: wrap at 72 characters
5. **Separate subject from body**: With a blank line

### Format Rules

#### Subject Line

- Start with an emoji
- Follow with type and optional scope
- Use the imperative, present tense: "change" not "changed" nor "changes"
- Don't capitalize the first letter of description
- No dot (.) at the end

#### Body (Optional)

- Use when the commit requires explanation
- Explain the motivation for the change
- Wrap at 72 characters
- Can use bullet points with `-` or `*`

#### Footer (Optional)

- Reference issues: `Closes #123`, `Fixes #456`
- Note breaking changes: `BREAKING CHANGE: description`
- Co-authors: `Co-authored-by: Name <email>`

## 💬 Commit Message Examples

### Simple Commits

```
✨ feat: add user profile page
```

```
🐛 fix: prevent race condition in data fetching
```

```
📚 docs: correct spelling in README
```

### Commits with Scope

```
✨ feat(auth): add password reset functionality
```

```
🐛 fix(ui): correct button alignment on mobile
```

```
⚡ perf(api): implement response caching
```

### Commits with Body

```
🐛 fix: prevent service timeout during upload

The previous timeout of 30 seconds was insufficient for large
file uploads. This commit increases the timeout to 5 minutes
and adds a progress indicator.

Closes #89
```

### Breaking Changes

```
💥 feat(api)!: change authentication to use JWT

BREAKING CHANGE: API now uses JWT tokens instead of session cookies.
All API clients must be updated to send Authorization header.

Migration guide:
- Update client to store JWT token
- Send token in Authorization header: `Bearer <token>`
- Remove cookie handling code
```

### Multiple Footers

```
🐛 fix: correct minor typos in code

See the issue for details on the typos fixed.

Reviewed-by: Alice <alice@example.com>
Refs #133
```

## 🚫 Bad Examples

```
Fix bug  // Missing emoji and type format, capitalized
```

```
feat: Add new feature  // Missing emoji
```

```
✨ feat Added new feature.  // Missing colon, capitalized, period
```

```
update code  // Missing emoji, vague, no type
```

```
✨ feat: add login, fix navigation, update styles  // Multiple changes
```

## 🔄 Workflow

1. **Make atomic changes** - One logical change per commit
2. **Choose correct emoji and type** - Select the most appropriate combination
3. **Add scope if helpful** - When it adds clarity
4. **Write clear description** - Explain what, not how
5. **Add body if needed** - For complex changes
6. **Reference issues** - Link related issues
7. **Mark breaking changes** - When applicable

## 🛠️ Tools

### Commitizen with Emoji Support

Interactive commit message helper:

```bash
npm install -g commitizen
npm install -g cz-conventional-changelog
echo '{ "path": "cz-conventional-changelog" }' > ~/.czrc
```

For emoji support:

```bash
npm install -g cz-emoji
echo '{ "path": "cz-emoji" }' > ~/.czrc
```

Usage:

```bash
git cz
```

### Commitlint Configuration

Enforce commit conventions with emoji support:

```javascript
// commitlint.config.js
module.exports = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "subject-case": [
      2,
      "never",
      ["sentence-case", "start-case", "pascal-case", "upper-case"],
    ],
    "header-pattern": [2, "always", /^[\u{1F300}-\u{1F9FF}][\s\S]*$/u],
  },
};
```

### Git Hook with Husky

```bash
npm install --save-dev husky
npx husky add .husky/commit-msg 'npx --no -- commitlint --edit "$1"'
```

### Semantic Release with Emoji

Configure semantic-release to parse emoji commits:

```json
{
  "plugins": [
    [
      "@semantic-release/commit-analyzer",
      {
        "preset": "conventionalcommits",
        "parserOpts": {
          "headerPattern": "^(\\p{Emoji}+)\\s+(\\w+)(?:\\(([\\w\\$\\.\\-\\*\\s]*)\\))?\\!?:\\s+(.+)$/u",
          "headerCorrespondence": ["emoji", "type", "scope", "subject"]
        }
      }
    ],
    "@semantic-release/release-notes-generator",
    "@semantic-release/github"
  ]
}
```

### Git Commit Template

Create `.gitmessage`:

```
# <emoji> <type>[optional scope]: <description>
#
# [optional body]
#
# [optional footer(s)]

# Emoji Guide:
# ✨ :sparkles: feat: New feature
# 🐛 :bug: fix: Bug fix
# 📚 :books: docs: Documentation
# 🎨 :art: style: Formatting
# ♻️ :recycle: refactor: Code refactoring
# ⚡ :zap: perf: Performance
# ✅ :white_check_mark: test: Testing
# 🏗️ :building_construction: build: Build system
# 👷 :construction_worker: ci: CI/CD
# 🧹 :broom: chore: Maintenance
# ⏪ :rewind: revert: Revert changes
```

Configure:

```bash
git config --global commit.template ~/.gitmessage
```

## 📖 References

- [Conventional Commits](https://www.conventionalcommits.org/) - Official specification
- [Gitmoji](https://gitmoji.dev/) - Emoji for Git commits
- [Angular Convention](https://github.com/angular/angular/blob/master/CONTRIBUTING.md#commit) - Convention that inspired Conventional Commits
- [Semantic Versioning](https://semver.org/) - How commits relate to version numbers
- [Commitizen](https://github.com/commitizen/cz-cli) - CLI helper for writing commits
- [Commitlint](https://commitlint.js.org/) - Lint commit messages
