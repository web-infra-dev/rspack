import { createRequire } from "module";

const req = createRequire(import.meta.url);

// `require()` is still bundled; `require.resolve` is preserved because
// requireResolve is disabled. In CommonJS output the preserved literal
// `import.meta.url` is a syntax error, so rspack emits a warning (warnings.js)
// and the bundle is not executed (test.config.js findBundle returns []).
export const value = req("./dep");
export const resolved = req.resolve("./dep");
