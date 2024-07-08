"use strict";

const { run } = require("../../utils/test-utils");

describe("loader error regression test for #1581", () => {
  it(`should not ignore loader's error produce a failing build`, async () => {
    // Ignoring assertion on stderr because ts-loader is producing depreciation warnings
    // with webpack@v5.0.0-beta.24 -> https://github.com/TypeStrong/ts-loader/issues/1169
    const { stdout, exitCode } = await run(__dirname, []);
    expect(exitCode).not.toEqual(0);
    expect(stdout).toContain("[1 error]");
    expect(stdout).toContain(`Cannot assign to 'foobar' because it is a constant`);
  });
});
