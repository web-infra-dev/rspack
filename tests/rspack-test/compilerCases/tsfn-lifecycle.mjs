import path from "node:path";
import { spawn } from "node:child_process";

function runChild(script, ...args) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, ["--expose-gc", script, ...args], {
      cwd: path.resolve(import.meta.dirname, "../../.."),
      stdio: ["ignore", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";
    let timedOut = false;
    const timeout = setTimeout(() => {
      timedOut = true;
      child.kill("SIGKILL");
    }, 20000);

    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });

    child.on("error", (error) => {
      clearTimeout(timeout);
      reject(error);
    });
    child.on("close", (code, signal) => {
      clearTimeout(timeout);
      if (code === 0 && !timedOut) {
        resolve(stdout);
        return;
      }
      reject(
        new Error(
          `${path.basename(script)} ${args.join(" ")}: ${timedOut ? "timed out after 20s" : `exited with code ${code}, signal ${signal}`}\n${stdout}\n${stderr}`,
        ),
      );
    });
  });
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    description:
      "should garbage collect hook closures that capture both compilation and compiler",
    async build() {
      await runChild(
        path.join(
          import.meta.dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-hooks.mjs",
        ),
      );
    },
  },
  {
    description:
      "should garbage collect option callbacks that capture both compilation and compiler",
    async build() {
      await runChild(
        path.join(
          import.meta.dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-options.mjs",
        ),
      );
    },
  },
  {
    description:
      "should keep option callbacks alive across multiple builds even after forced gc",
    async build() {
      await runChild(
        path.join(
          import.meta.dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-options-multiple-builds.mjs",
        ),
      );
    },
  },
  {
    description:
      "should garbage collect chunks after compiler is garbage collected",
    async build() {
      await runChild(
        path.join(
          import.meta.dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-chunk.mjs",
        ),
      );
    },
  },
  {
    description:
      "should garbage collect module graph connections from a previous build",
    async build() {
      await runChild(
        path.join(
          import.meta.dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-module-graph-connection.mjs",
        ),
      );
    },
  },
  {
    description:
      "should garbage collect custom runtime modules after compiler close",
    async build() {
      await runChild(
        path.join(
          import.meta.dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-runtime-module.mjs",
        ),
      );
    },
  },
  {
    description: "should report a clear error when APIs are called after close",
    async build() {
      await runChild(
        path.join(
          import.meta.dirname,
          "fixtures",
          "tsfn-lifecycle",
          "closed-compiler-error.mjs",
        ),
      );
    },
  },
  ...["lifecycle", "rebuild", "failure", "no-emit", "detector"].map((mode) => ({
    description: `should detect libuv handle leaks: ${mode}`,
    async build() {
      await runChild(
        path.join(
          import.meta.dirname,
          "fixtures/tsfn-lifecycle/check-libuv-handles.mjs",
        ),
        mode,
      );
    },
  })),
  {
    description: "should not allocate libuv async handles per loader module",
    async build() {
      const script = path.join(
        import.meta.dirname,
        "fixtures/tsfn-lifecycle/check-libuv-handles.mjs",
      );
      const small = JSON.parse(await runChild(script, "scale", "4"));
      const large = JSON.parse(await runChild(script, "scale", "64"));
      expect(large.asyncDelta).toBe(small.asyncDelta);
    },
  },
];
