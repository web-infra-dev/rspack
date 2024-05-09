# E2E

The `packages/playground` provides e2e testing feature. We use [playwright](https://github.com/Microsoft/playwright) as the e2e testing framework.

## Cases

The entry point of a test case is a file ending with `.test.ts`, and the parent directory of this file is the project directory.

Here are some rules about test cases:

* The project directory must contain `rspack.config.js` to start the dev server.
* The project directory can contain multi `*.test.ts`.
* All test cases share dependencies, so just add dependencies in `packages/playground/package.json`.
* The cases folder should contain the category folders and then is the project folders. In principle, there should be no third-level directory.

## Fixtures

The `fixtures` is a feature of playwright, in short it provides a variable that is generated in before{Each|All} and destroyed in after{Each|All}. More information see [test-fixtures](https://playwright.dev/docs/test-fixtures)

Here are some rules when defining a new fixture:

* Private fixtures should start with `_` and are used only in the current file.
* A file only provides fixtures with the same name.
* A file can only provide one option and starts with `default`
* Register fixtures in `fixtures/index.ts` and export only necessary variables and types.

Here are some existing fixtures:

#### pathInfo

This fixture will generate test environment, and calculate the PathInfo.
``` ts
type PathInfo = {
    // test file path
    testFile: string;
    // project dir
    testProjectDir: string
    // temporary project directory to be copied into
    tempProjectDir: string
}
```

#### rspack

This fixture will start the rspack dev server and provide some useful methods.
``` ts
type Rspack = {
    // rspack running project directory
    projectDir: string
    // rspack compiler
    compiler: Compiler
    // rspack dev server
    devServer: DevServer
    // waiting for rspack build finish
    waitingForBuild: () => Promise<void>
    // waiting for hmr finish, the poll function is used to check
    waitingForHmr: (poll: () => Promise<boolean>) => Promise<void>
}
```

#### fileAction

This fixture will provide file change operations.
``` ts
type fileAction = {
    updateFile(relativePath: string, fn: (content: string) => string): void
    deleteFile(relativePath: string): void
}
```

## How it works

* playwright scan all test case and allocates a worker to run each case.
* `pathInfo` copy the project directory corresponding to the current case to `temp/${worker_index}`.
* `rspack` rewrite dev server port to `8000 + worker_index` and start compiler and dev server in `temp/${worker_index}`.
* run current tests.
* `rspack` close dev server and compiler.
* `pathInfo` clear `temp/${worker_index}`

