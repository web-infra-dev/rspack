// `import.meta.url` must continue to work alongside dynamic-import rewriting.
const here = import.meta.url;

const dir = process.env.x;

// Template literal — should be rewritten with origin appended.
const a = import(`./translations/${dir}/strings.json`);

// Variable specifier with attributes — origin must be inserted *after* the
// attributes object as the third argument.
const name = process.env.name;
const b = import(name, { with: { type: 'json' } });

// Literal specifier — should fall through to rspack's default static path
// (no origin injection).
const c = import('./literal.js');

// Nested require inside the import() arg — must still be collected as a
// dependency by the parser even though we short-circuit ImportParserPlugin.
const d = import(require('./nested.js').name);

// `webpackIgnore: true` — must NOT be rewritten; the user wants the native
// runtime to load it.
const e = import(/* webpackIgnore: true */ `./ignored/${dir}.mjs`);

// Shadow-resistance: even when `undefined` is shadowed in the enclosing
// scope, the missing-attributes placeholder must still resolve to the
// real `undefined` value (`void 0`).
function shadowSafe(undefined) {
  return import(`./translations/${dir}/strings.json`);
}
const f = shadowSafe(42);

console.log(here, a, b, c, d, e, f);
