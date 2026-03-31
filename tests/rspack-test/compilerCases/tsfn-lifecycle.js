const path = require("node:path");
const { spawn } = require("node:child_process");
const { createFsFromVolume, Volume } = require("memfs");

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
    description: "should reject run after compiler.close",
    options() {
      return {
        mode: "development",
        entry: "./c",
      };
    },
    async compiler(context, compiler) {
      compiler.outputFileSystem = createFsFromVolume(new Volume());
    },
    async build(context, compiler) {
      await new Promise((resolve, reject) => {
        compiler.run(err => {
          if (err) return reject(err);
          resolve();
        });
      });

      await new Promise((resolve, reject) => {
        compiler.close(err => {
          if (err) return reject(err);
          resolve();
        });
      });
    },
  },
  {
    description: "should release tsfn-owned compiler and compilation lifecycles after close",
    async build() {
      await runChild(
        path.join(
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "gc-check.cjs",
        ),
      );
    },
  },
  {
    description: "should release tsfn-owned option callback lifecycles after close",
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
    description: "should isolate compilation-scoped tsfn lifecycles across compilers on the same js thread",
    async build() {
      await runChild(
        path.join(
          __dirname,
          "fixtures",
          "tsfn-lifecycle",
          "parallel-compilers.cjs",
        ),
      );
    },
  },
];
