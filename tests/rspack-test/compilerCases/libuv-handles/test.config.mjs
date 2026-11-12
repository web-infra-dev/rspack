import assert from "node:assert/strict";
import { runNodeCase } from "@rspack/test-tools/helper/node-case";

// Measure all libuv handles in a clean Node event loop, including unref handles.
// These assertions guard resource counts, not latency, CPU or memory thresholds.
/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  ...[
    {
      name: "compiler-close",
      description: "should release handles after each compiler close",
    },
    {
      name: "compiler-rebuild",
      description:
        "should not accumulate async handles across rebuilds and release them after close",
    },
    {
      name: "loader-error",
      description:
        "should release handles after closing a compiler with loader errors",
    },
    {
      name: "emit-skipped",
      description: "should release handles after close when emit is skipped",
    },
    {
      name: "unref-detection",
      description:
        "should detect unreferenced handles and recover after they are closed",
    },
  ].map(({ name, description }) => ({
    name,
    description,
    async run() {
      await runNodeCase(new URL(`./${name}.mjs`, import.meta.url));
    },
  })),
  {
    name: "module-count",
    description:
      "should allocate the same number of async handles for 4 and 64 modules",
    async run() {
      const scenario = new URL("./module-count.mjs", import.meta.url);
      const small = await runNodeCase(scenario, [4]);
      const large = await runNodeCase(scenario, [64]);
      assert.equal(large.asyncDelta, small.asyncDelta);
    },
  },
];
