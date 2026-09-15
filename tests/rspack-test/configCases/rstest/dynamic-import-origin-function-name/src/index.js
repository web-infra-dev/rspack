const dir = process.env.x;

// Template literal — exercises the dynamic-import rewrite path. The callee
// must come from `injectDynamicImportOrigin: { functionName }`, NOT from
// `output.importFunctionName` (left at its `'import'` default in this fixture).
const a = import(`./translations/${dir}/strings.json`);

console.log(a);
