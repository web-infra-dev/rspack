import path from "node:path";
import { createCompilerCase, describeByWalk } from "@rspack/test-tools";
import { findTestFile } from "@rspack/test-tools/helper/read-test-file";

const srcDir = path.resolve(import.meta.dirname, "./fixtures");

// Both walks share the outer suite while preserving each case's describe block.
function describeCase(name, register) {
  if (name === "compiler") {
    register();
  } else {
    describe(name, register);
  }
}

describe("compiler", () => {
  describeByWalk(
    import.meta.filename,
    (name, testConfig, dist) => {
      createCompilerCase(name, srcDir, dist, testConfig);
    },
    { level: 1, type: "file", describe: describeCase },
  );

  describeByWalk(
    import.meta.filename,
    (name, src, dist) => {
      const testConfig = findTestFile(src, "test.config");
      if (testConfig) {
        createCompilerCase(name, src, dist, testConfig);
      }
    },
    { level: 1, type: "directory", describe: describeCase },
  );
});
