'use strict';
// A named export reassigned the same way: `exports.foo` must survive too.
Object.defineProperty(exports, '__esModule', { value: true });
exports.foo = void 0;
var foo = 7;
exports.foo = foo;
