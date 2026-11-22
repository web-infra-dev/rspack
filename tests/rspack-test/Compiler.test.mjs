import fs from "node:fs";
import path from "node:path";
import { createCompilerCase, describeByWalk } from "@rspack/test-tools";
import { findTestFile } from "@rspack/test-tools/helper/read-test-file";

const srcDir = path.resolve(import.meta.dirname, "./fixtures");
const casesDir = path.join(import.meta.dirname, "compilerCases");
const watcherCasesDir = path.join(casesDir, "native-watcher");
const [shardIndex, shardCount] = (
  process.env.RSPACK_COMPILER_SHARD || "0/1"
).split("/").map(Number);
const caseEntries = new Set(
  [
    ...fs.readdirSync(casesDir).filter((entry) => entry !== "native-watcher"),
    ...fs.readdirSync(watcherCasesDir)
      .filter((group) => !group.startsWith("_"))
      .flatMap((group) => fs.readdirSync(path.join(watcherCasesDir, group))),
  ]
    .sort()
    .filter((_, index) => index % shardCount === shardIndex),
);

// All walks share the outer suite and shard by case file, not by category.
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
      if (!caseEntries.has(path.basename(testConfig))) return;
      createCompilerCase(name, srcDir, dist, testConfig);
    },
    { level: 1, type: "file", describe: describeCase },
  );

  describeByWalk(
    import.meta.filename,
    (name, src, dist) => {
      if (!caseEntries.has(path.basename(src))) return;
      const testConfig = findTestFile(src, "test.config");
      if (testConfig) {
        createCompilerCase(name, src, dist, testConfig);
      }
    },
    { level: 1, type: "directory", describe: describeCase },
  );

  describeByWalk(
    import.meta.filename,
    (name, testConfig, dist) => {
      if (!caseEntries.has(path.basename(testConfig))) return;
      createCompilerCase(name, srcDir, dist, testConfig);
    },
    {
      level: 3,
      type: "file",
      exclude: [/^(?!native-watcher(?:\/|$))/],
      describe: describeCase,
    },
  );
});
