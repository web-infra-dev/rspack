'use strict';
// Standard `@babel/preset-env` output shape for `export default <expr>`:
// the default is initialised to `void 0`, then reassigned through a chained
// `var _default = (exports.default = value)` whose local is never read.
// Regression guard for https://github.com/web-infra-dev/rspack/issues/14589
Object.defineProperty(exports, '__esModule', { value: true });
exports['default'] = void 0;
var value = 42;
var _default = (exports['default'] = value);
