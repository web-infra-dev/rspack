# Sending a Pull Request

1. [Fork](https://help.github.com/articles/fork-a-repo/) the Rspack repository into your own GitHub account.
2. [Clone](https://help.github.com/articles/cloning-a-repository/) the repository to your local.
3. Checkout a new branch from `main`.
4. Set up the development environment, you can read the "Setup Development Environment" section below to learn about it.
5. If you've fixed a bug or added code that should be tested, then add some tests.
6. Make sure all the tests pass, you can read the "Testing" section below to learn about it.
7. Run `pnpm run lint:js` and `pnpm run lint:rs` to check the code style.
8. If you've changed some Node.js packages, you should add a new [changeset](https://github.com/changesets/changesets). Run `pnpm run changeset`, select the changed packages and add the changeset info.
9. If you've changed some Rust packages, you should add a new [changeset](https://github.com/changesets/changesets) for `@rspack/binding` package.
10. Submit the Pull Request, make sure all CI runs pass.
11. The maintainers will review your Pull Request soon.

When submitting a Pull Request, please note the following:

- Keep your PRs small enough, so that each PR only addresses a single issue or adds a single feature.
- Please include an appropriate description in the PR, and link related issues.

## Format of PR titles

The format of PR titles follow Conventional Commits.

A example

```
feat(ui): Add `Button` component
^    ^    ^
|    |    |__ Subject
|    |_______ Scope
|____________ Type
```

Your PR

- must have a `Type`
- Optionally have a `Scope`
  - `Scope` should be lower case
- must have a `Subject`
