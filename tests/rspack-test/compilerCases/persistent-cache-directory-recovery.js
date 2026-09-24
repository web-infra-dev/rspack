const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const CASE_NAME = "persistent-cache-directory-recovery";

const FRESH_PROCESS_SCRIPT = String.raw`
const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const rspack = require(process.argv[1]);
const options = JSON.parse(process.argv[2]);
const resultFile = process.argv[3];
const outputFile = process.argv[4];
const unchangedFile = process.argv[5];
const builtModules = [];

const compiler = rspack({
  ...options,
  plugins: [
    ...(Array.isArray(options.plugins) ? options.plugins : []),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap("RecordBuiltModules", compilation => {
          compilation.hooks.buildModule.tap("RecordBuiltModules", module => {
            builtModules.push(module.resource);
          });
        });
      }
    }
  ]
});

compiler.run((error, stats) => {
  const statsError =
    !error && stats && stats.hasErrors()
      ? new Error(stats.toString({ all: false, errors: true, errorDetails: true }))
      : null;
  const result = {
    builtModules,
    unchangedModuleBuilt: builtModules.includes(unchangedFile),
    runtimeOutput: fs.existsSync(outputFile)
      ? execFileSync(process.execPath, [outputFile], { encoding: "utf8" }).trim()
      : null,
    cacheLog: stats
      ? stats.toString({
          all: false,
          colors: false,
          logging: false,
          loggingDebug: /^rspack\.persistentCache$/
        })
      : ""
  };

  compiler.close(closeError => {
    const finalError = error || statsError || closeError;
    result.error = finalError ? String(finalError.stack || finalError) : null;
    fs.writeFileSync(resultFile, JSON.stringify(result, null, 2));
    if (finalError) {
      console.error(finalError);
      process.exitCode = 1;
    }
  });
});
`;

let root;
let sourceDirectory;
let outputDirectory;
let cacheDirectory;
let valueFile;
let unchangedFile;
let outputFile;
let compilerOptions;
let cleanupWaiter;
let openFds;

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

function listFiles(directory) {
  if (!fs.existsSync(directory)) return [];

  const files = [];
  const visit = (currentDirectory) => {
    for (const entry of fs.readdirSync(currentDirectory, {
      withFileTypes: true,
    })) {
      const file = path.join(currentDirectory, entry.name);
      if (entry.isDirectory()) {
        visit(file);
      } else {
        files.push(path.relative(directory, file));
      }
    }
  };
  visit(directory);
  return files.sort();
}

function cacheState(label) {
  const files = listFiles(cacheDirectory);
  const committedFiles = files.filter(
    (file) => !file.split(path.sep).includes(".temp"),
  );
  const packs = committedFiles.filter((file) => file.endsWith(".pack"));
  const metadata = committedFiles.filter(
    (file) => path.basename(file) === "_meta",
  );
  return {
    label,
    files,
    committedFiles,
    packs,
    metadata,
    packsMissingMetadata: packs.filter(
      (pack) => !metadata.includes(path.join(path.dirname(pack), "_meta")),
    ),
  };
}

function waitForTempCleanup() {
  return new Promise((resolve, reject) => {
    const waiter = (error) => {
      clearTimeout(timeout);
      if (cleanupWaiter === waiter) cleanupWaiter = undefined;
      if (error) {
        reject(error);
      } else {
        resolve();
      }
    };
    const timeout = setTimeout(
      () => {
        if (cleanupWaiter === waiter) cleanupWaiter = undefined;
        reject(
          new Error(
            `Timed out waiting for ${path.join(cacheDirectory, ".temp")} cleanup`,
          ),
        );
      },
      process.platform === "win32" ? 30_000 : 15_000,
    );
    cleanupWaiter = waiter;
  });
}

function closeDescriptors() {
  let closeError;
  for (const fd of openFds) {
    try {
      fs.closeSync(fd);
      openFds.delete(fd);
    } catch (error) {
      if (!closeError) closeError = error;
    }
  }
  return closeError;
}

function createIntermediateFileSystem() {
  cleanupWaiter = undefined;
  openFds = new Set();
  return {
    ...fs,
    // The bridge currently consumes the returned buffer without using bytesRead.
    read(fd, options, callback) {
      fs.read(fd, options, (error, bytesRead, buffer) => {
        if (error) return callback(error, bytesRead, buffer);
        callback(error, bytesRead, buffer.subarray(0, bytesRead));
      });
    },
    open(file, flags, callback) {
      fs.open(file, flags, (error, fd) => {
        if (!error) openFds.add(fd);
        callback(error, fd);
      });
    },
    close(fd, callback) {
      fs.close(fd, (error) => {
        if (!error) openFds.delete(fd);
        callback(error);
      });
    },
    rmdir(directory, callback) {
      fs.rmdir(directory, (error) => {
        const relativeDirectory = path.relative(cacheDirectory, directory);
        if (
          path.basename(directory) === ".temp" &&
          relativeDirectory !== "" &&
          !relativeDirectory.startsWith(`..${path.sep}`) &&
          relativeDirectory !== ".."
        ) {
          // Legacy cache streams do not explicitly close their JavaScript descriptors.
          // Release them before returning to Rust; subsequent metadata refresh has not started.
          const closeError = closeDescriptors();
          const effectiveError = error || closeError;
          const waiter = cleanupWaiter;
          if (waiter) waiter(effectiveError || null);
          return callback(effectiveError);
        }
        callback(error);
      });
    },
  };
}

function run(compiler, changes) {
  const cleanup = waitForTempCleanup();
  return new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      cleanup.then(() => {
        if (error) return reject(error);
        if (!stats) return reject(new Error("Compiler run returned no stats"));
        if (stats.hasErrors()) {
          return reject(
            new Error(
              stats.toString({ all: false, errors: true, errorDetails: true }),
            ),
          );
        }
        resolve(stats);
      }, reject);
    }, changes);
  });
}

function assertPersisted(state) {
  if (state.packs.length === 0 || state.packsMissingMetadata.length > 0) {
    throw new Error(
      `Expected committed persistent cache after ${state.label}: ${JSON.stringify(state)}`,
    );
  }
}

async function runStage(compiler, label, value, changes) {
  write(valueFile, `export default ${JSON.stringify(value)};\n`);
  await run(compiler, changes);
  const runtimeOutput = execFileSync(process.execPath, [outputFile], {
    encoding: "utf8",
  }).trim();
  expect(runtimeOutput).toBe(`${value} unchanged`);
  const state = cacheState(label);
  assertPersisted(state);
}

function buildFreshProcess() {
  const resultFile = path.join(root, "fresh-process.json");
  execFileSync(
    process.execPath,
    [
      "-e",
      FRESH_PROCESS_SCRIPT,
      require.resolve("@rspack/core"),
      JSON.stringify(compilerOptions),
      resultFile,
      outputFile,
      unchangedFile,
    ],
    {
      cwd: sourceDirectory,
      stdio: "pipe",
      windowsHide: true,
    },
  );
  return JSON.parse(fs.readFileSync(resultFile, "utf8"));
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
module.exports = {
  description:
    "should recreate persistent cache files when a bucket directory is deleted during rebuild",
  options(context) {
    root = context.getDist(CASE_NAME);
    sourceDirectory = path.join(root, "src");
    outputDirectory = path.join(root, "dist");
    cacheDirectory = path.join(root, "cache");
    valueFile = path.join(sourceDirectory, "value.js");
    unchangedFile = path.join(sourceDirectory, "unchanged.js");
    outputFile = path.join(outputDirectory, "main.js");

    fs.rmSync(root, { recursive: true, force: true });
    write(
      path.join(sourceDirectory, "index.js"),
      'import value from "./value.js";\nimport unchanged from "./unchanged.js";\nconsole.log(value, unchanged);\n',
    );
    write(valueFile, 'export default "A";\n');
    write(unchangedFile, 'export default "unchanged";\n');

    compilerOptions = {
      context: sourceDirectory,
      mode: "development",
      devtool: false,
      target: "node",
      entry: "./index.js",
      cache: {
        type: "persistent",
        portable: false,
        version: CASE_NAME,
        storage: {
          type: "filesystem",
          directory: cacheDirectory,
        },
      },
      experiments: {
        newCache: false,
      },
      output: {
        path: outputDirectory,
        filename: "main.js",
        clean: true,
      },
    };
    return compilerOptions;
  },
  compiler(_context, compiler) {
    compiler.outputFileSystem = fs;
    compiler.intermediateFileSystem = createIntermediateFileSystem();
  },
  async build(context, compiler) {
    const modifiedValue = () => ({ modifiedFiles: new Set([valueFile]) });
    let buildError;
    try {
      await runStage(compiler, "initial build", "A");
      await runStage(compiler, "ordinary rebuild", "B", modifiedValue());

      fs.rmSync(cacheDirectory, { recursive: true, force: true });
      await runStage(
        compiler,
        "first rebuild after deleting cache",
        "C",
        modifiedValue(),
      );

      fs.rmSync(cacheDirectory, { recursive: true, force: true });
      await runStage(
        compiler,
        "second rebuild after deleting cache",
        "D",
        modifiedValue(),
      );
    } catch (error) {
      buildError = error;
    }

    let compilerCloseError;
    try {
      await context.getCompiler().close();
    } catch (error) {
      compilerCloseError = error;
    }
    const descriptorError = closeDescriptors();
    if (buildError) throw buildError;
    if (compilerCloseError) throw compilerCloseError;
    if (descriptorError) throw descriptorError;

    const fresh = buildFreshProcess();
    context.setValue("freshProcess", fresh);
  },
  check({ context }) {
    if (context.hasError()) throw context.getError()[0];
    const fresh = context.getValue("freshProcess");
    expect(fresh.runtimeOutput).toBe("D unchanged");
    expect(fresh.cacheLog).toContain(
      "make persistent cache recovery succeeded",
    );
    expect(fresh.unchangedModuleBuilt).toBe(false);
    expect(fresh.error).toBe(null);
  },
};
