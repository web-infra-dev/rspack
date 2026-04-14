const path = require("node:path");
const { spawn } = require("node:child_process");

function runChild(script) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, ["--expose-gc", script], {
      cwd: path.resolve(__dirname, "../../.."),
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
      "should clear compiler-scoped callbacks after compiler.close",
    async build() {
      await runChild(
        path.join(
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "close-clears-callbacks.cjs",
        ),
      );
    },
  },
];
