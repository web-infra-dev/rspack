# Testing

We currently have two sets of test suites, one for Rust and one for Node.js.

## Rust Testing

- `cargo test` will run all the rust side tests, which includes standalone tests for core functionality and plugins.
- `UPDATE=1 cargo test` will automatically update the failed snapshot

## Node Testing

We are maintaining two test suites for Node Testing in Rspack, Rspack Testing and Webpack Testing

### Webpack Testing

We copy the whole webpack test suites into [webpack-test](https://github.com/web-infra-dev/rspack/tree/main/webpack-test#progressively-migrate-webpack-test) folder to check the compatibility with webpack. If you add features or fix bugs we recommend you check whether this feature or bug is covered in webpack test suites first. If it's covered and testable in Webpack Testing, you can enable specific test case by setting return value to true in [`test.filter.js`](https://github.com/web-infra-dev/rspack/blob/80e97477483fcb912473ae339c37d5a5e247f7b1/webpack-test/cases/compile/error-hide-stack/test.filter.js#L2C33-L2C84) in this case folder to enable this case. See more details in https://github.com/web-infra-dev/rspack/blob/main/webpack-test/README.md, Please note that don't modify original test code in Webpack Testing, if you find difficulties in running test suites without modifying original code, you can copy this test code in the following \[Rspack Testing\](#Rspack Testing).

#### Run Tests

```sh
# In root path
./x build -a # build binding and js part
./x test webpack # run webpack test suites
```

### Rspack Testing

We maintain test suites in Rspack Testing which is not coverable or need to be modified in Webpack Testing. The test suites lies in [rspack-test](https://github.com/web-infra-dev/rspack/tree/main/packages/rspack/tests). This folder structure is similar with Webpack Testing.

#### Run Tests

```sh
# In root path
./x build -a
./x test js
```

Or only test the package that you made the changes:

```sh
# In the Node.js package path
pnpm run build && pnpm run test
```

To update snapshots:

```sh
pnpm --filter '@rspack/*' test -- -u
```

### Node Testing Suite Overview

We use jest for Node.js tests, The most important test cases are the case in the `packages/rspack`. most of these cases comes from webpack https://github.com/webpack/webpack/tree/main/test because we want to make sure that Rspack can work as same as webpack.

There are three kinds of integration cases in `@rspack/core`.

#### case.test.ts

Cases are used to test normal build behavior, we use these cases to test against bundler core functionality, like `entry`, `output`, `module` `resolve`, etc. it will first build your test file to test whether the input could be compiled successfully, then it will use the bundled test file to run test cases in the test file to test bundler's all kinds of behavior.

#### configCase.test.ts

Cases are used to test custom build behavior, you could use custom `webpack.config.js` to override default build behavior, you can use these cases to test against behavior related to specific config.

##### statsTestCase.test.ts

Cases are used to test your stats, By Default we will use jest's snapshot to snapshot your stats, and we **highly** recommend to **avoid** snapshot except statsCase. you can use statsCase to test behaviors like code splitting | bundle splitting, which is hard to test by just running code.

## E2E

The `packages/playground` provides e2e testing feature. We use [playwright](https://github.com/Microsoft/playwright) as the e2e testing framework.

### Cases

The entry point of a test case is a file ending with `.test.ts`, and the parent directory of this file is the project directory.

Here are some rules about test cases:

- The project directory must contain `rspack.config.js` to start the dev server.
- The project directory can contain multi `*.test.ts`.
- All test cases share dependencies, so just add dependencies in `packages/playground/package.json`.
- The cases folder should contain the category folders and then is the project folders. In principle, there should be no third-level directory.

### Fixtures

The `fixtures` is a feature of playwright, in short it provides a variable that is generated in before{Each|All} and destroyed in after{Each|All}. More information see [test-fixtures](https://playwright.dev/docs/test-fixtures)

Here are some rules when defining a new fixture:

- Private fixtures should start with `_` and are used only in the current file.
- A file only provides fixtures with the same name.
- A file can only provide one option and starts with `default`
- Register fixtures in `fixtures/index.ts` and export only necessary variables and types.

Here are some existing fixtures:

#### pathInfo

This fixture will generate test environment, and calculate the PathInfo.

```ts
type PathInfo = {
  // test file path
  testFile: string;
  // project dir
  testProjectDir: string;
  // temporary project directory to be copied into
  tempProjectDir: string;
};
```

#### rspack

This fixture will start the rspack dev server and provide some useful methods.

```ts
type Rspack = {
  // rspack running project directory
  projectDir: string;
  // rspack compiler
  compiler: Compiler;
  // rspack dev server
  devServer: DevServer;
  // waiting for rspack build finish
  waitingForBuild: () => Promise<void>;
  // waiting for hmr finish, the poll function is used to check
  waitingForHmr: (poll: () => Promise<boolean>) => Promise<void>;
};
```

#### fileAction

This fixture will provide file change operations.

```ts
type fileAction = {
  updateFile(relativePath: string, fn: (content: string) => string): void;
  deleteFile(relativePath: string): void;
};
```

### How it works

- playwright scan all test case and allocates a worker to run each case.
- `pathInfo` copy the project directory corresponding to the current case to `temp/${worker_index}`.
- `rspack` rewrite dev server port to `8000 + worker_index` and start compiler and dev server in `temp/${worker_index}`.
- run current tests.
- `rspack` close dev server and compiler.
- `pathInfo` clear `temp/${worker_index}`
