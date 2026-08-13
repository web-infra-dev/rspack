const path = require("path");
const { ResolverFactory } = require("../../napi");

let referencesDisabled = new ResolverFactory({
  extensions: [".js", ".ts"],
  tsconfig: {
    configFile: path.resolve("./app/tsconfig.json")
    // references disabled by default
  }
});

let referencesEnabled = new ResolverFactory({
  extensions: [".js", ".ts"],
  tsconfig: {
    configFile: path.resolve("./app/tsconfig.json"),
    references: "auto"
  }
});

let referencesDisabledResolveResult = referencesDisabled.sync(
  path.resolve("./component/src/index.ts"),
  "foo"
);
let referencesEnabledResolveResult = referencesEnabled.sync(
  path.resolve("./component/src/index.ts"),
  "foo"
);

console.log(referencesDisabledResolveResult);
// { path: '.../examples/tsconfig_references/app/mock_foo/index.js' }
console.log(referencesEnabledResolveResult);
