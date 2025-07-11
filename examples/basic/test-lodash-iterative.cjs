const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
const fs = require('fs');

// Test on the already optimized lodash chunk
const lodashCode = fs.readFileSync('./dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.local-optimized.js', 'utf8');
const shareUsage = JSON.parse(fs.readFileSync('./dist/share-usage.json', 'utf8'));

// Generate tree-shaking configuration for lodash-es
const lodashUsage = shareUsage.consume_shared_modules['lodash-es'];
const lodashConfig = {};

lodashUsage.used_exports.forEach(exportName => { lodashConfig[exportName] = true; });
lodashUsage.unused_exports.forEach(exportName => { lodashConfig[exportName] = false; });
lodashUsage.possibly_unused_exports.forEach(exportName => { lodashConfig[exportName] = false; });

const treeShakeConfig = { treeShake: { 'lodash-es': lodashConfig } };

console.log('=== LODASH ITERATIVE TREE-SHAKING TEST ===');
console.log('Original size:', lodashCode.length);
console.log('Tree-shaking config exports:', Object.keys(lodashConfig).length);
console.log('Used exports:', lodashUsage.used_exports.length);
console.log('Unused exports:', lodashUsage.unused_exports.length);

console.log('\n=== FIRST PASS ===');
const firstPass = optimize(lodashCode, JSON.stringify(treeShakeConfig));
console.log('First pass size:', firstPass.length);
console.log('First pass reduction:', lodashCode.length - firstPass.length, 'bytes');

console.log('\n=== SECOND PASS ===');
const secondPass = optimize(firstPass, JSON.stringify(treeShakeConfig));
console.log('Second pass size:', secondPass.length);
console.log('Second pass reduction:', firstPass.length - secondPass.length, 'bytes');

console.log('\n=== THIRD PASS ===');
const thirdPass = optimize(secondPass, JSON.stringify(treeShakeConfig));
console.log('Third pass size:', thirdPass.length);
console.log('Third pass reduction:', secondPass.length - thirdPass.length, 'bytes');

// Test with empty config to see if webpack tree-shaking alone finds anything
console.log('\n=== WEBPACK TREE-SHAKING ONLY TEST ===');
const webpackOnly = optimize(lodashCode, JSON.stringify({ treeShake: {} }));
console.log('Webpack-only size:', webpackOnly.length);
console.log('Webpack-only reduction:', lodashCode.length - webpackOnly.length, 'bytes');

// Test the opposite - run webpack tree-shaking on already macro-optimized code
console.log('\n=== MACRO THEN WEBPACK TEST ===');
const webpackAfterMacro = optimize(firstPass, JSON.stringify({ treeShake: {} }));
console.log('Webpack after macro size:', webpackAfterMacro.length);
console.log('Additional webpack reduction:', firstPass.length - webpackAfterMacro.length, 'bytes');

console.log('\n=== CONVERGENCE ANALYSIS ===');
if (firstPass.length === secondPass.length && secondPass.length === thirdPass.length) {
    console.log('✅ Full convergence: All passes identical');
} else if (secondPass.length === thirdPass.length) {
    console.log('✅ Convergence after first pass');
} else {
    console.log('⚠️  Still optimizing after multiple passes');
}

console.log('\n=== TOTAL OPTIMIZATION COMPARISON ===');
console.log('Single macro pass:', lodashCode.length - firstPass.length, 'bytes');
console.log('Iterative macro:', lodashCode.length - thirdPass.length, 'bytes');
console.log('Webpack-only:', lodashCode.length - webpackOnly.length, 'bytes');
console.log('Macro + Webpack:', lodashCode.length - webpackAfterMacro.length, 'bytes');