"use strict";

const { runWatch } = require("../../utils/test-utils");

describe('"analyze" option', () => {
  it("should load webpack-bundle-analyzer plugin with --analyze flag", async () => {
    const { stderr, stdout } = await runWatch(__dirname, ["--analyze"], {
      killString: /Webpack Bundle Analyzer is started at/,
    });

    expect(stderr).toBeFalsy();
    expect(stdout).toContain("Webpack Bundle Analyzer is started at");
  });
});
