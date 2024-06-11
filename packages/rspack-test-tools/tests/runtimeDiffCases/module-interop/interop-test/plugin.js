/**
 * This test came from https://github.com/sokra/interop-test
 * So this plugin is used for generating test cases from modules dir
 */

const path = require("path")
const fs = require("fs")

const src = path.resolve(__dirname, "src");

const comparator = (a, b) => {
  a = a.replace(/\.m?jso?n?$/, "");
  b = b.replace(/\.m?jso?n?$/, "");
  if (a.startsWith(b)) return 1;
  if (b.startsWith(a)) return -1;
  return a < b ? -1 : a === b ? 0 : 1;
};

const print = (expr) =>
  `Promise.resolve().then(() => {}).then(() => console.log(util.inspect(${expr}, { showHidden: true, breakLength: Infinity, compact: true, getters: true })))`;

const cases = (filename) => [
  ["import x", `import x from "${filename}"; ${print("x")};`],
  [
    "import { default as x }",
    `import { default as x } from "${filename}"; ${print("x")};`,
  ],
  [
    "import * as x; x.default",
    `import * as x from "${filename}"; ${print("x.default")};`,
  ],
  [
    "import * as x; ident(x).default",
    `import * as x from "${filename}"; ${print("Object(x).default")};`,
  ],
  [
    "import { named as x }",
    `import { named as x } from "${filename}"; ${print("x")};`,
  ],
  [
    "import * as x; x.named",
    `import * as x from "${filename}"; ${print("x.named")};`,
  ],
  [
    "import * as x; ident(x).named",
    `import * as x from "${filename}"; ${print("Object(x).named")};`,
  ],
  [
    "import { __esModule as x }",
    `import { __esModule as x } from "${filename}"; ${print("x")};`,
  ],
  [
    "import * as x; x.__esModule",
    `import * as x from "${filename}"; ${print("x.__esModule")};`,
  ],
  [
    "import * as x; ident(x).__esModule",
    `import * as x from "${filename}"; ${print("Object(x).__esModule")};`,
  ],
  ["import * as x", `import * as x from "${filename}"; ${print("x")};`],
  [
    "import()",
    `import("${filename}").then(x => { ${print(
      "x"
    )}; }).catch(err => { console.error(err); process.exitCode = 1; });`,
  ],
  [
    "x = require(); x.default",
    `const x = require("${filename}"); ${print("x.default")};`,
  ],
  [
    "x = require(); ident(x).default",
    `const x = require("${filename}"); ${print("Object(x).default")};`,
  ],
  [
    "{ named } = require()",
    `const { named } = require("${filename}"); ${print("named")};`,
  ],
  [
    "x = require(); x.named",
    `const x = require("${filename}"); ${print("x.named")};`,
  ],
  [
    "x = require(); ident(x).named",
    `const x = require("${filename}"); ${print("Object(x).named")};`,
  ],
  [
    "{ __esModule } = require()",
    `const { __esModule } = require("${filename}"); ${print(
      "__esModule"
    )};`,
  ],
  [
    "x = require(); x.__esModule",
    `const x = require("${filename}"); ${print("x.__esModule")};`,
  ],
  [
    "x = require(); ident(x).__esModule",
    `const x = require("${filename}"); ${print("Object(x).__esModule")};`,
  ],
  ["x = require()", `const x = require("${filename}"); ${print("x")};`],
  [
    "await import() === require()",
    `import("${filename}").then(x => { const y = require("${filename}"); ${print(
      "x === y"
    )}; }).catch(err => { console.error(err); process.exitCode = 1; });`,
  ],
  [
    "import * as x; x === await import()",
    `import * as x from "${filename}"; import("${filename}").then(y => { ${print(
      "x === y"
    )}; }).catch(err => { console.error(err); process.exitCode = 1; });`,
  ],
];

module.exports = class GenerateCasesPlugin {
  constructor(ext = ".mjs") {
    this.ext = ext
  }

  apply(compiler) {
    const ext = this.ext;
    const modules = fs
      .readdirSync(path.resolve(src, "../modules"))
      .filter((name) => !name.startsWith("_"))
      .filter(
        (item, idx, array) =>
          item.endsWith(ext) || !array.includes(item.replace(/\.m?js$/g, ext))
      )
      .sort(comparator);
    const extEntryFile = path.resolve(src, `index${ext}`);
    const extDir = path.resolve(src, ext);
    compiler.hooks.beforeCompile.tapPromise(GenerateCasesPlugin.name, async () => {
      await fs.promises.mkdir(extDir);
      await Promise.all(modules.map(moduleName => fs.promises.mkdir(path.resolve(extDir, moduleName))));
      const imports = await Promise.all(
        modules.flatMap(moduleName => {
          return cases(path.resolve(src, `../modules/${moduleName}`)).map(
            async ([name, content], idx) => {
              const requireTest = name.includes("require()");
              const testFilename = `index${idx}${requireTest ? ".js" : ext}`;
              const testPath = path.resolve(extDir, moduleName, testFilename);
              await fs.promises.writeFile(
                testPath,
                requireTest
                  ? 'const util = require("util");\n' + content + "\n"
                  : 'import util from "util";\n' + content + "\nexport {};"
              );
              return `import ${JSON.stringify(testPath)};\n`
            }
          )
        })
      );
      await fs.promises.writeFile(
        extEntryFile,
        imports
      );
    })
    compiler.hooks.done.tapPromise(GenerateCasesPlugin.name, async () => {
      await fs.promises.rm(extEntryFile);
      await fs.promises.rm(extDir, { recursive: true });
    });
  }
}
