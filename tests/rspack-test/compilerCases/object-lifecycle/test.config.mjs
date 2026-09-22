/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    name: "hooks",
    description:
      "should collect hook closures and captured compiler/compilations",
    async run() {
      const { default: run } = await import("./hooks.mjs");
      await run();
    },
  },
  {
    name: "options",
    description:
      "should collect option callbacks and captured compiler/compilations",
    async run() {
      const { default: run } = await import("./options.mjs");
      await run();
    },
  },
  {
    name: "options-multiple-builds",
    description: "should keep option callbacks alive across builds and GC",
    async run() {
      const { default: run } = await import("./options-multiple-builds.mjs");
      await run();
    },
  },
  {
    name: "chunks",
    description: "should collect chunks and chunk groups after compiler close",
    async run() {
      const { default: run } = await import("./chunks.mjs");
      await run();
    },
  },
  {
    name: "module-graph-connection",
    description: "should collect connections from the previous build",
    async run() {
      const { default: run } = await import("./module-graph-connection.mjs");
      await run();
    },
  },
  {
    name: "runtime-module",
    description: "should collect custom runtime modules after compiler close",
    async run() {
      const { default: run } = await import("./runtime-module.mjs");
      await run();
    },
  },
];
