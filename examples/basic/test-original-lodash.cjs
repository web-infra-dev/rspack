const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
const fs = require('fs');

// Test on the original unoptimized lodash chunk  
const lodashCode = fs.readFileSync('./dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js', 'utf8');
const shareUsage = JSON.parse(fs.readFileSync('./dist/share-usage.json', 'utf8'));

// Generate tree-shaking configuration for lodash-es
const lodashUsage = shareUsage.consume_shared_modules['lodash-es'];
const lodashConfig = {};

lodashUsage.used_exports.forEach(exportName => { lodashConfig[exportName] = true; });
lodashUsage.unused_exports.forEach(exportName => { lodashConfig[exportName] = false; });
lodashUsage.possibly_unused_exports.forEach(exportName => { lodashConfig[exportName] = false; });

const treeShakeConfig = { treeShake: { 'lodash-es': lodashConfig } };

console.log('=== ORIGINAL LODASH ITERATIVE TEST ===');
console.log('Original size:', lodashCode.length, 'bytes');

// Test single pass vs iterative
console.log('\n=== OUR ITERATIVE IMPLEMENTATION ===');
const start = performance.now();
const iterativeResult = optimize(lodashCode, JSON.stringify(treeShakeConfig));
const iterativeTime = performance.now() - start;

console.log('Iterative result size:', iterativeResult.length, 'bytes');
console.log('Iterative reduction:', lodashCode.length - iterativeResult.length, 'bytes');
console.log('Iterative time:', iterativeTime.toFixed(2), 'ms');
console.log('Iterative reduction %:', ((lodashCode.length - iterativeResult.length) / lodashCode.length * 100).toFixed(2), '%');

// Test webpack tree-shaking only (no macro transformations)
console.log('\n=== WEBPACK TREE-SHAKING ONLY ===');
const webpackStart = performance.now();
const webpackResult = optimize(lodashCode, JSON.stringify({ treeShake: {} }));
const webpackTime = performance.now() - webpackStart;

console.log('Webpack-only size:', webpackResult.length, 'bytes');
console.log('Webpack-only reduction:', lodashCode.length - webpackResult.length, 'bytes');
console.log('Webpack-only time:', webpackTime.toFixed(2), 'ms');
console.log('Webpack-only reduction %:', ((lodashCode.length - webpackResult.length) / lodashCode.length * 100).toFixed(2), '%');

// Test if running multiple times on original gives same result
console.log('\n=== CONVERGENCE TEST ===');
const secondRun = optimize(lodashCode, JSON.stringify(treeShakeConfig));
const thirdRun = optimize(iterativeResult, JSON.stringify(treeShakeConfig));

console.log('Second run same as first:', iterativeResult.length === secondRun.length);
console.log('Third run (on optimized) same as first:', iterativeResult.length === thirdRun.length);

if (iterativeResult.length === thirdRun.length) {
    console.log('✅ Perfect convergence achieved');
} else {
    console.log('⚠️  Additional optimization possible:', iterativeResult.length - thirdRun.length, 'bytes');
}

// Compare with our known good result  
const knownOptimized = fs.readFileSync('./dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.local-optimized.js', 'utf8');
console.log('\n=== COMPARISON WITH KNOWN OPTIMIZED ===');
console.log('Our iterative result:', iterativeResult.length, 'bytes');  
console.log('Known optimized result:', knownOptimized.length, 'bytes');
console.log('Difference:', Math.abs(iterativeResult.length - knownOptimized.length), 'bytes');

if (iterativeResult.length === knownOptimized.length) {
    console.log('✅ Our iterative implementation matches known optimized result exactly');
} else if (iterativeResult.length < knownOptimized.length) {
    console.log('🎉 Our iterative implementation is MORE optimized than known result!');
} else {
    console.log('⚠️  Known result is more optimized - potential improvement needed');
}