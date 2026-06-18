import { createRequire } from "module";

// None of these created requires is used as a real require object inside this module (no
// `.resolve`, no value/member access) — they only escape via exports. They must still NOT be
// cleared to `undefined`, because importers call `.resolve(...)` on the exported values.
// `finish()` resolves each exported local binding to its declaration through the scope-aware
// createRequire tag, which covers every form below (aliases included) without name guessing.

// Plain named export.
export const req = createRequire(import.meta.url);

// Aliased export: the public export name is `exportedAlias`, the local binding is `aliased`.
const aliased = createRequire(import.meta.url);
export { aliased as exportedAlias };

// Default-aliased export: the public name is `default`, the local binding is `def`.
const def = createRequire(import.meta.url);
export { def as default };

// Declarator value-copy then export: `copy` carries `original`'s declaration via the tag.
const original = createRequire(import.meta.url);
const copy = original;
export { copy };

// Assignment value-copy then export: `assigned = source` also carries `source`'s declaration
// via the tag, so exporting `assigned` keeps `source` (a declarator copy is not required).
const source = createRequire(import.meta.url);
let assigned;
assigned = source;
export { assigned };
