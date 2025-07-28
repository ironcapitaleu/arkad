# Contributing to This Project

Thank you for considering contributing.
From bug reports and feature requests to documentation improvements and code, we welcome all kinds of contributions.

## How to Contribute
- Report bugs via [issues](https://github.com/DeCa09/arkad/issues/new)
- Propose features via [issues](https://github.com/DeCa09/arkad/issues/new)
- Submit code via [pull requests](https://github.com/DeCa09/arkad/pulls)

## Submitting Code via Pull Requests
Thank you very much for wanting to add to this project's codebase. By submitting a pull request, you agree that your contribution will be licensed under the same licenses as this project.

### Getting Started
Make sure to change the **placeholders** in the commands.
1. **Fork** the repository and **clone** it locally:
   ```bash
   git clone https://github.com/your-username/your-project.git
   cd your-project
   ```
2. Check if everything works by running the project:
   ```bash
   cargo run
   ```
3. Create a new branch for your changes:
   ```bash
   git checkout -b your-feature-name
   ```
### Code guidelines
Make sure the code guidelines outlined [here](https://docs.google.com/document/d/15cCsiH9SULp1dKRUhxhNc5mDCajH6r4uaeoqiJyRVcQ/) are followed at all times. This saves us a lot of time.

### Submitting a Pull Request
1. Make sure your branch is up to date:
   ```bash
   git pull origin main
   ```
2. Check if all tests, formatting checks and linting checks pass:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```
3. If everything passes, push your changes to your own branch:
   ```bash
   git push origin your-feature-name
   ```
4. Open a Pull Request with a clear title and description


