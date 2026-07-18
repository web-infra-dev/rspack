const {
  experiments: { VirtualModulesPlugin },
} = require('@rspack/core');

const dependencyCount = 8;
const keysPerDependency = 1;
const importNames = [];
const expectedValues = [];
const virtualModules = {};

for (let dependency = 0; dependency < dependencyCount; dependency++) {
  const properties = [];
  for (let key = 0; key < keysPerDependency; key++) {
    const name = `checked${dependency}_${key}`;
    const value = dependency * keysPerDependency + key;
    importNames.push(name);
    expectedValues.push(value);
    properties.push(`\t${name}: ${value}`);
  }
  virtualModules[`checked-source-${dependency}.js`] = `const values = {
${properties.join(',\n')}
};

for (const key in values) Object(exports)[key] = values[key];
`;
}

virtualModules['checked-barrel.js'] = Array.from(
  { length: dependencyCount },
  (_, dependency) => `export * from "./checked-source-${dependency}";`,
).join('\n');

virtualModules['checked-entry.js'] = `import {
\t${importNames.join(',\n\t')}
} from "./checked-barrel";

const getGeneratedModule = (source, request) => {
\tconst start = source.indexOf(\`"\${request}"\`);
\texpect(start).toBeGreaterThanOrEqual(0);
\tconst remaining = source.slice(start);
\tconst end = remaining.indexOf('\\n},\\n"');
\treturn end < 0 ? remaining : remaining.slice(0, end);
};

it("should preserve checked reexport semantics", () => {
\texpect([
\t\t${importNames.join(',\n\t\t')}
\t]).toEqual(${JSON.stringify(expectedValues)});
});

it("should share one checked reexport runtime across eight large dependencies", () => {
\tconst fs = require("fs");
\tconst path = require("path");
\tconst entrySource = fs.readFileSync(
\t\tpath.join(__dirname, "./checked-reexport.js"),
\t\t"utf-8"
\t);
\tconst runtimeSource = fs.readFileSync(
\t\tpath.join(__dirname, "./checked-reexport-runtime.js"),
\t\t"utf-8"
\t);
\tconst barrelSource = getGeneratedModule(entrySource, "./checked-barrel.js");
\tconst runtime = barrelSource.includes("__rspack_context.cr(")
\t\t? "__rspack_context"
\t\t: "__webpack_require__";

\texpect(barrelSource.split(\`\${runtime}.cr(\`)).toHaveLength(9);
\texpect(barrelSource).not.toContain(\`\${runtime}.cr =\`);
\texpect(runtimeSource.split(\`\${runtime}.cr =\`)).toHaveLength(2);
});
`;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  entry: {
    'checked-reexport': {
      import: './checked-entry.js',
      runtime: 'checked-reexport-runtime',
    },
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  node: false,
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    minimize: false,
    usedExports: true,
    concatenateModules: false,
  },
  plugins: [new VirtualModulesPlugin(virtualModules)],
};
