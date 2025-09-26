import value from "./cjsFile.cjs";
export { value }

export function foo() {
  console.log(module);
  if (module.children) {
    module.children = module.children.filter((item) => item.filename !== path);
  }
}

export function bar() {
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

export function baz() {
  if (module.exports && module.exports.test) {
    return module.exports.test();
  }

  return undefined;
}

