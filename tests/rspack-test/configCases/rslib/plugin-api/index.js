import value from "./cjsFile.cjs";
export { value }

console.log(require.cache)
console.log(require.extensions)
console.log(require.config)
console.log(require.version)
console.log(require.include)
console.log(require.onError)
console.log(value);

export function filterModuleChildren() {
  console.log(module);
  if (module.children) {
    module.children = module.children.filter((item) => item.filename !== path);
  }
}

export function rewriteModuleExports() {
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

export function callModuleMethod() {
  if (typeof module === "undefined") {
    return undefined;
  }

  if (module.exports && module.exports.test) {
    return module.exports.test();
  }

  return undefined;
}

export const moduleType = typeof module;
