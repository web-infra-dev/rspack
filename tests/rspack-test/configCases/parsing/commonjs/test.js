const path = require('path');
const fs = require('fs');

const file = path.resolve(__dirname, 'bundle0.js')
const content = fs.readFileSync(file, 'utf-8');

it ('`module` var should be reserved as-is', () => {
	expect(content).toContain(`function foo() {
  console.log(module);
  if (module.children) {
    module.children = module.children.filter((item) => item.filename !== path);
  }
}

function bar() {
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

function baz() {
  if (module.exports && module.exports.test) {
    return module.exports.test();
  }

  return undefined;
}`)
	const exported = require('./bundle0.js')
	expect(exported.value).toBe(42)
})


// it ('`module` should be treated as a member', () => {
// 	expect(content).toContain(``)
// 	const exported = require('./bundle1.mjs')
// 	expect(exported.value).toBe(42)
// })
