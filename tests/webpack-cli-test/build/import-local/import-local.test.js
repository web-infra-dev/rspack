"use strict";

const { run } = require("../../utils/test-utils");

const importLocalMock = jest.fn();
jest.setMock("import-local", importLocalMock);

describe("import local", () => {
  beforeEach(() => {
    importLocalMock.mockClear();
  });
  it("should skip import local when supplied", async () => {
    const { exitCode, stderr, stdout } = await run(__dirname, [], {
      env: { WEBPACK_CLI_SKIP_IMPORT_LOCAL: true },
    });
    expect(importLocalMock).toHaveBeenCalledTimes(0);
    expect(exitCode).toBe(0);
    expect(stderr).toBeFalsy();
    expect(stdout).toBeTruthy();
  });
});
