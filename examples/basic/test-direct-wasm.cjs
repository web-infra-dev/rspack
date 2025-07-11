#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');

// Create test chunks to understand the issue
const tests = [
  {
    name: 'Standard webpack bundle',
    code: `
var __webpack_modules__ = {
  100: function(module, exports, __webpack_require__) {
    __webpack_require__(200);
  },
  200: function(module, exports) {
    exports.foo = "bar";
  },
  300: function(module, exports) {
    exports.unused = "should be removed";
  }
};
__webpack_require__(100);
`
  },
  {
    name: 'Split chunk format',
    code: `
(self["webpackChunk"] = self["webpackChunk"] || []).push([["test"], {
  "mod1": function(module, exports, __webpack_require__) {
    __webpack_require__("mod2");
  },
  "mod2": function(module, exports) {
    exports.foo = "bar";
  },
  "mod3": function(module, exports) {
    exports.unused = "should be removed";
  }
}]);
`
  }
];

console.log('=== TESTING WEBPACK TREE SHAKING ===\n');

for (const test of tests) {
  console.log(`--- ${test.name} ---`);
  console.log('Input size:', test.code.length, 'bytes');
  
  try {
    const result = optimize(test.code, JSON.stringify({
      enable_webpack_tree_shaking: true
    }));
    
    console.log('Output size:', result.length, 'bytes');
    console.log('Size change:', result.length - test.code.length, 'bytes');
    
    // Check what remains
    if (test.name.includes('Standard')) {
      console.log('Module 300 removed:', !result.includes('300:'));
    } else {
      console.log('Module mod3 removed:', !result.includes('mod3'));
    }
    
    // Write output for inspection
    const filename = test.name.toLowerCase().replace(/ /g, '-') + '.js';
    fs.writeFileSync(filename, result);
    console.log('Output written to:', filename);
    
  } catch (err) {
    console.error('Error:', err.message);
  }
  
  console.log();
}

// Now test the actual lodash chunk with minimal config
console.log('--- Lodash chunk with ALL exports false ---');
const lodashChunk = fs.readFileSync(
  path.join(__dirname, 'dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js'),
  'utf-8'
);

// Get all export names from share-usage.json
const shareUsage = JSON.parse(fs.readFileSync(path.join(__dirname, 'dist/share-usage.json'), 'utf-8'));
const lodashData = shareUsage.consume_shared_modules['lodash-es'];
const allExports = [
  ...lodashData.used_exports,
  ...lodashData.unused_exports,
  ...lodashData.possibly_unused_exports
];

console.log('Total exports:', allExports.length);
console.log('Setting all exports to false...');

const allFalseConfig = {
  lodash_usage: {
    used_exports: [],
    unused_exports: allExports,
    possibly_unused_exports: []
  },
  enable_webpack_tree_shaking: true
};

const allFalseResult = optimize(lodashChunk, JSON.stringify(allFalseConfig));
console.log('Result size:', allFalseResult.length, 'bytes');
console.log('Modules remaining:', (allFalseResult.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+"\s*:/g) || []).length);

// Check if main lodash module exists
const hasMainModule = allFalseResult.includes('lodash-es/lodash.js');
console.log('Main lodash module exists:', hasMainModule);

// Check for any export definitions
const exportDefs = (allFalseResult.match(/__webpack_require__\.d\(__webpack_exports__/g) || []).length;
console.log('Export definitions found:', exportDefs);