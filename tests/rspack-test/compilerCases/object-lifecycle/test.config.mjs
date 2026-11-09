import path from "node:path";
import { spawn } from "node:child_process";

function runChild(script, args = []) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, ["--expose-gc", script, ...args], {
      cwd: path.resolve(import.meta.dirname, "../../../.."),
      stdio: ["ignore", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", chunk => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", chunk => {
      stderr += chunk.toString();
    });

    child.on("error", reject);
    child.on("close", code => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(
        new Error(
          stderr ||
          stdout ||
          `GC lifecycle script exited with code ${code}`,
        ),
      );
    });
  });
}

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
    name: "import-module",
    description: "should collect importModule publicPath callbacks before compiler close",
    async run() {
      const { default: run } = await import("./import-module.mjs");
      await run();
    },
  },
  {
    name: "runtime-module-multiple-builds",
    description: "should collect old runtime modules while the compiler remains open",
    async run() {
      const { default: run } = await import("./runtime-module-multiple-builds.mjs");
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
  {
    description: "should finish queued loader tasks when dynamic worker startup fails",
    async run() {
      await runChild(path.join(import.meta.dirname, "worker-startup-failure.mjs"), ["--dynamic"]);
    },
  },
  {
    description: "should recover queued loader tasks after the only worker exits",
    async run() {
      await runChild(path.join(import.meta.dirname, "parallel-loader-dispatch.mjs"), ["--single"]);
    },
  },
  {
    description: "should release transferred task subscriptions when native compiler construction fails",
    async run() {
      await runChild(path.join(import.meta.dirname, "js-task-constructor-failure.mjs"));
    },
  },
  {
    description: "should release shared subscriptions and worker registrations with their final owner",
    async run() {
      await runChild(path.join(import.meta.dirname, "js-task-subscription-api.mjs"));
    },
  },
  {
    description: "should release task subscriptions with unsafeFastDrop",
    async run() {
      await runChild(path.join(import.meta.dirname, "js-task-subscriptions.mjs"), ["--parallel", "--unsafe-fast-drop"]);
    },
  },
  {
    description: "should isolate loader worker startup failures from main-only compilers",
    async run() {
      await runChild(path.join(import.meta.dirname, "worker-startup-failure.mjs"));
    },
  },
  {
    description: "should recreate worker subscriptions after all compilers are collected",
    async run() {
      await runChild(path.join(import.meta.dirname, "js-task-subscriptions.mjs"), ["--parallel"]);
    },
  },
  {
    description: "should share JS task subscriptions and recreate them after compiler GC",
    async run() {
      await runChild(path.join(import.meta.dirname, "js-task-subscriptions.mjs"));
    },
  },
  {
    description: "should release loader additionalData handles after compiler close",
    async run() {
      await runChild(path.join(import.meta.dirname, "gc-check-loader-additional-data.mjs"));
    },
  },
  {
    description: "should exit with idle native loader workers",
    async run() {
      await runChild(path.join(import.meta.dirname, "parallel-loader-process-exit.mjs"));
    },
  },
  {
    description: "should release additionalData after the main object owner env is recreated",
    async run() {
      await runChild(path.join(import.meta.dirname, "main-env-recreation.mjs"));
    },
  },
  {
    description: "should distribute native loader tasks and recover from worker exit",
    async run() {
      await runChild(path.join(import.meta.dirname, "parallel-loader-dispatch.mjs"));
    },
  },
];
