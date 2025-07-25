>**Note**
> This package is heavily based on [webpack/test](https://github.com/webpack/webpack/tree/main/test)

# Progressively migrate webpack test

Originally, we use this formula to calculate the compatibility `passedTestCaseCount / totalTestCount`, totalTestCount = passedTestCaseCount + failedTestCount + skippedTestCount , but sometimes it maybe hard to compatible with all webpack test cases for some reasons (e.g. performance, legacy feature that we don't want to support), we need a method to skip these tests that we will not support. Thus, we adjust the original formula to `(passedTestCaseCount + willNotSupportTestCount) / totalTestCount`.

Currently, we use a `test.filter.js` under each failed test case directory to skip failed test case, using this method could let us migrate webpack test case progressively without affect the real compatibility (Because this method will not influence the real `passedTestCaseCount`).
e.g.

```js
// test.filter.js
module.exports = () => {
    return false; // false means this testcase is skipped for now, but maybe we will support in the future, `-1` means this test case we don't want to compatible with, this related to `willNotSupportTest`.
}

```

When you find that we have passed some failed testcases which is skipped for now, you could change the `test.filter.js` to

```js
module.exports = () => {
    return true
}
```

## Wasm test
See [@rspack/test-tools](../../packages/rspack-test-tools)


or delete the `test.filter.js`

# Welcome to the webpack test suite

Every pull request that you submit to webpack (besides README and spelling corrections in comments) requires tests that are created.

But don't give up hope!!! Although our tests may appear complex and overwhelming, once you become familiar with the test suite and structure, adding and creating tests will be fun and beneficial as you work inside the codebase! ❤

## tl;dr

Run all tests (this automatically runs the setup):

```sh
pnpm test
```

Run an individual suite:

```sh
pnpm jest ConfigTestCases
```

Watch mode:

```sh
pnpm jest --watch ConfigTestCases
```

See also: [Jest CLI docs](https://jestjs.io/docs/cli)

## Test suite overview

We use Jest for our tests. For more information on Jest you can visit their [homepage](https://jestjs.io/)!

### Class tests

All test files can be found in *.test.js. There are many tests that simply test APIs of a specific class/file (such as `Compiler`, `Errors`, Integration, `Parser`, `RuleSet`, Validation).
If the feature you are contributing involves one of those classes, then best to start there to understand the structure.

### xCases

In addition to Class specific tests, there are also directories that end in "Cases". The suites for these cases also have corresponding *.test.js files.

#### cases (`TestCases.test.js`) <sup>1</sup>

Cases are a set of general purpose tests that will run against a variety of permutations of webpack configurations. When you are making a general purpose change that doesn't require you to have a special configuration, you would likely add your tests here. Inside of the `./test/cases` directory you will find tests are broken into thematic sub directories. Take a moment to explore the different options.

To add a new case, create a new directory inside of the top level test groups, and then add an `index.js` file (and any other supporting files).

By default this file will be the entry point for the test suite and you can add your `it()`'s there. This will also become bundled so that node env support happens as well.

#### configCases (`ConfigTestCases.basictest.js`) <sup>1</sup>

If you are trying to solve a bug which is reproducible when x and y properties are used together in a config, then configCases is the place to be!!!!

In addition to an `index.js`, these configCases require a `webpack.config.js` is located inside of your test suite. This will run this specific config through `webpack` just as you were building individually. They will use the same loading/bundling technique of your `it()` tests, however you now have a more specific config use cases that you can write even before you start coding.

#### statsCases (`StatsTestCases.basictest.js`)

Stats cases are similar to configCases except specifically focusing on the `expected` output of your stats. Instead of writing to the console, however the output of stats will be written to disk.

By default, the "expected" outcome is a pain to write by hand so instead when statsCases are run, runner is checking output using jest's awesome snapshot functionality.

Basically you don't need to write any expected behaviors yourself. The assumption is that the stats output from your test code is what you expect.

Please follow the approach described below:

* write your test code in `statsCases/` folder by creating a separate folder for it, for example `statsCases/some-file-import-stats/index.js`

```javascript
import("./someModule");
```

* don't forget the `webpack.config.js`
* run the test
* jest will automatically add the output from your test code to `StatsTestCases.test.js.snap` and you can always check your results there
* Next time test will run -> runner will compare results against your output written to snapshot previously

You can read more about SnapShot testing [right here](https://jestjs.io/docs/snapshot-testing)

## Questions? Comments?

If you are still nervous or don't quite understand, please submit an issue and tag us in it, and provide a relevant PR while working on!

## Footnotes

<sup>1</sup> webpack's parser supports the use of ES2015 features like arrow functions, ESM exports, etc. However as a library we follow Node.js' timeline for dropping older versions of node. Because of this we expect your tests on GitHub Actions to pass all the way back to NodeJS v10; Therefore if you would like specific tests that use these features to be ignored if they are not supported, then you should add a `test.filter.js` file. This allows you to import the syntax needed for that test, meanwhile ignoring it on node versions (during CI) that don't support it. webpack has a variety of helpful examples you can refer to if you are just starting out. See the `./helpers` folder to find a list of the versions.

## Credits

Thanks to:

* [The webpack team and community](https://webpack.js.org/) for creating a great bundler and ecosystem from which we draw a lot of inspiration.
* [@sokra](https://github.com/sokra) for the great work on the [webpack](https://github.com/webpack/webpack) project.
