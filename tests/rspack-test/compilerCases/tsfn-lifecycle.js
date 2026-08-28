const path = require("node:path");
const { spawn } = require("node:child_process");

function runChild(script, timeout, expectedStdout) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, ["--expose-gc", script], {
      cwd: path.resolve(__dirname, "../../.."),
      stdio: ["ignore", "pipe", "pipe"],
      timeout,
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
        if (expectedStdout && !stdout.includes(expectedStdout)) {
          reject(
            new Error(
              `Lifecycle script exited before producing ${JSON.stringify(expectedStdout)}`,
            ),
          );
          return;
        }
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
module.exports = [
  {
    description:
      "should garbage collect hook closures that capture both compilation and compiler",
    async build() {
      await runChild(
        path.join(
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-hooks.cjs",
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
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-options.cjs",
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
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-options-multiple-builds.cjs",
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
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-chunk.cjs",
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
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-module-graph-connection.cjs",
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
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-runtime-module.cjs",
        ),
      );
    },
  },
  {
    description: "should report a clear error when APIs are called after close",
    async build() {
      await runChild(
        path.join(
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "closed-compiler-error.cjs",
        ),
      );
    },
  },
  {
    description:
      "should let the process exit after running a parallel loader",
    async build() {
      await runChild(
        path.join(
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "parallel-loader-process-exit.cjs",
        ),
        15_000,
        "parallel-loader-complete",
      );
    },
  },
  {
    description:
      "should release main registry data when the native additional data handle is dropped",
    async build() {
      await runChild(
        path.join(
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check-loader-additional-data.cjs",
        ),
        15_000,
      );
    },
  },
];
