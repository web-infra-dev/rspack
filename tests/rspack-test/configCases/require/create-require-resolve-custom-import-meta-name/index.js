import { createRequire } from "module";

// requireResolve is disabled, so these created requires are kept. The kept argument must
// honor a customized output.importMetaName instead of emitting a literal `import.meta`, for
// the deferred variable form AND the non-deferred forms below.
const req = createRequire(import.meta.url);
export const resolved = req.resolve("path");

// Non-deferred: a multi-argument createRequire keeps its literal call (a clear cannot drop
// the extra argument's side effect), so the customized name must be applied to its argument.
let ran = false;
const reqMulti = createRequire(import.meta.url, (ran = true));
export const resolvedMulti = reqMulti.resolve("path");

// Non-deferred: an inline createRequire used as an exported value is kept verbatim.
export default createRequire(import.meta.url);
