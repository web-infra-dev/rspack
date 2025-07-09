const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');

const splitChunk = `
"use strict";
(self["webpackChunktest"] = self["webpackChunktest"] || []).push([["chunk"], {
  "module1": function(module, exports, __webpack_require__) {
    console.log("Module 1");
    __webpack_require__("module2");
  },
  "module2": function(module, exports, __webpack_require__) {
    console.log("Module 2");
  },
  "module3": function(module, exports, __webpack_require__) {
    console.log("Module 3 - unreachable");
  }
}]);
`;

console.log('Testing split chunk parsing...');
console.error('Input length:', splitChunk.length);

const result = optimize(splitChunk, JSON.stringify({
  enable_webpack_tree_shaking: true
}));

console.error('Result length:', result.length);
console.error('Module 3 removed:', !result.includes('Module 3'));