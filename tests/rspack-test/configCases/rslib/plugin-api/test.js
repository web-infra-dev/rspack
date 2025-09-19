const path = require('path');
const fs = require('fs');

const file = path.resolve(__dirname, 'bundle0.js')
const content = fs.readFileSync(file, 'utf-8');

it ('some expressions should not be handled by APIPlugin', () => {
	expect(content).toContain(`
console.log(require.cache)
console.log(require.extensions)
console.log(require.config)
console.log(require.version)
console.log(require.include)
console.log(require.onError)
console.log((_cjsFile_cjs__WEBPACK_IMPORTED_MODULE_0___default()));

function filterModuleChildren() {
  console.log(module);
  if (module.children) {
    module.children = module.children.filter((item) => item.filename !== path);
  }
}

function rewriteModuleExports() {
  if (typeof module === "undefined") {
    return undefined;
  }

  const original = module.exports;

  try {
    module.exports = {
      ...module.exports,
      test: () => "ok"
    };

    return module.exports.test();
  } finally {
    module.exports = original;
  }
}

function callModuleMethod() {
  if (typeof module === "undefined") {
    return undefined;
  }

  if (module.exports && module.exports.test) {
    return module.exports.test();
  }

  return undefined;
}

const moduleType = typeof module;
`)
	const exported = require('./bundle0.js')
	expect(exported.value).toBe(42)
})
