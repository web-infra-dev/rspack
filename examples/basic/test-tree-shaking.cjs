const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
const fs = require('fs');

const testCode = fs.readFileSync('./test-lodash-simulation.js', 'utf8');

console.log('=== LODASH SIMULATION TEST ===');
console.log('Original code size:', testCode.length);

// Count original modules
const originalModules = (testCode.match(/"[^"]+"/g) || []).map(m => m.replace(/"/g, '')).sort();
console.log('Original modules:', originalModules.join(', '));

// Test with empty config (no macro transformations, just webpack tree-shaking)
const emptyConfig = { treeShake: {} };

console.log('\nRunning webpack tree-shaking only (no macro transformations)...');
const result = optimize(testCode, JSON.stringify(emptyConfig));

console.log('Optimized code size:', result.length);
console.log('Reduction:', testCode.length - result.length, 'bytes');

// Count remaining modules  
const resultModules = (result.match(/"[^"]+"/g) || []).map(m => m.replace(/"/g, '')).sort();
console.log('Remaining modules:', resultModules.join(', '));

const removedModules = originalModules.filter(m => !resultModules.includes(m));
console.log('Removed modules:', removedModules.join(', '));

console.log('\n=== OPTIMIZED RESULT ===');
console.log(result);