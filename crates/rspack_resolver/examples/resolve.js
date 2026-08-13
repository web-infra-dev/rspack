#!/usr/bin/env node
// Usage: node examples/resolve.js <absolute_dir> <specifier>
const path = require("path");
const fs = require("fs");
const { ResolverFactory, CachedInputFileSystem } = require("enhanced-resolve");

function printUsageAndExit() {
  console.error("Usage: node examples/resolve.js <absolute_dir> <specifier>");
  process.exit(1);
}

const [, , baseDir, specifier] = process.argv;
if (!baseDir || !specifier) printUsageAndExit();
if (!path.isAbsolute(baseDir)) {
  console.error(`${baseDir} must be an absolute path.`);
  process.exit(1);
}
if (!fs.existsSync(baseDir) || !fs.statSync(baseDir).isDirectory()) {
  console.error(
    `${baseDir} must be a directory that will be resolved against.`
  );
  process.exit(1);
}

console.log("path:", baseDir);
console.log("specifier:", specifier);

const fileDependencies = new Set();
const missingDependencies = new Set();

const resolver = ResolverFactory.createResolver({
  aliasFields: ["browser"],
  alias: { asdf: "./test.js" },
  extensions: [".js", ".ts"],
  extensionAlias: { ".js": [".ts", ".js"] },
  conditionNames: ["node", "import"], // ESM
  // conditionNames: ['node', 'require'], // CJS
  fileSystem: new CachedInputFileSystem(fs, 4000)
});

resolver.resolve(
  {},
  baseDir,
  specifier,
  { fileDependencies, missingDependencies },
  (err, result, resDetails) => {
    if (err) {
      console.log("Error:", err.message || err);
    } else {
      console.log("Resolved:", result);
    }
    console.log("file_deps:", Array.from(fileDependencies).sort());
    console.log("missing_deps:", Array.from(missingDependencies).sort());
  }
);
