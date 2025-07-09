const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
const fs = require('fs');

console.log('=== TESTING ALL EXPORTS FALSE ===');

// Test on original lodash chunk
const originalCode = fs.readFileSync('./dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js', 'utf8');
const shareUsage = JSON.parse(fs.readFileSync('./dist/share-usage.json', 'utf8'));

console.log('Original size:', originalCode.length, 'bytes');

// Count original modules and requires
const originalModules = (originalCode.match(/"[^"]+"\s*:\s*function/g) || []).length;
const originalRequires = (originalCode.match(/__webpack_require__\(/g) || []).length;
const originalPure = (originalCode.match(/\/\* #__PURE__ \*\//g) || []).length;

console.log('Original modules:', originalModules);
console.log('Original __webpack_require__ calls:', originalRequires);  
console.log('Original PURE annotations:', originalPure);

// Create config with ALL exports set to false
const lodashUsage = shareUsage.consume_shared_modules['lodash-es'];
const allFalseConfig = {};

// Set ALL exports to false (including the used ones)
[...lodashUsage.used_exports, ...lodashUsage.unused_exports, ...lodashUsage.possibly_unused_exports]
    .forEach(exportName => { allFalseConfig[exportName] = false; });

const treeShakeConfig = { 
    treeShake: { 
        'lodash-es': allFalseConfig 
    } 
};

console.log('\nTree-shake config: ALL', Object.keys(allFalseConfig).length, 'exports set to FALSE');
console.log('This should remove ALL lodash functionality...');

console.log('\n=== RUNNING OPTIMIZATION WITH ALL FALSE ===');
const result = optimize(originalCode, JSON.stringify(treeShakeConfig));

console.log('Result size:', result.length, 'bytes');
console.log('Reduction:', originalCode.length - result.length, 'bytes');
console.log('Reduction %:', ((originalCode.length - result.length) / originalCode.length * 100).toFixed(2), '%');

// Count what remains
const resultModules = (result.match(/"[^"]+"\s*:\s*function/g) || []).length;
const resultRequires = (result.match(/__webpack_require__\(/g) || []).length;
const resultPure = (result.match(/\/\* #__PURE__ \*\//g) || []).length;

console.log('\nRemaining modules:', resultModules);
console.log('Remaining __webpack_require__ calls:', resultRequires);
console.log('Remaining PURE annotations:', resultPure);

console.log('\nReductions:');
console.log('- Modules removed:', originalModules - resultModules);
console.log('- Require calls removed:', originalRequires - resultRequires);
console.log('- PURE annotations removed:', originalPure - resultPure);

// Analyze what's left
console.log('\n=== ANALYZING WHAT REMAINS ===');
if (result.length < 1000) {
    console.log('Small result - showing full content:');
    console.log('===');
    console.log(result);
    console.log('===');
} else {
    console.log('Large result - showing first 1000 chars:');
    console.log('===');
    console.log(result.substring(0, 1000) + '...');
    console.log('===');
}

// Test if we can shake even more with webpack-only
console.log('\n=== TESTING WEBPACK-ONLY ON ALL-FALSE RESULT ===');
const webpackOnly = optimize(result, JSON.stringify({ treeShake: {} }));
console.log('Webpack-only further reduction:', result.length - webpackOnly.length, 'bytes');

if (webpackOnly.length < result.length) {
    console.log('🔍 Webpack tree-shaking found additional dead code!');
    
    const finalModules = (webpackOnly.match(/"[^"]+"\s*:\s*function/g) || []).length;
    const finalRequires = (webpackOnly.match(/__webpack_require__\(/g) || []).length;
    
    console.log('Final modules after webpack:', finalModules);
    console.log('Final requires after webpack:', finalRequires);
    
    if (webpackOnly.length < 500) {
        console.log('\nFinal tiny result:');
        console.log('===');
        console.log(webpackOnly);
        console.log('===');
    }
} else {
    console.log('✅ No additional optimization possible with webpack tree-shaking');
}

// Compare with completely empty config
console.log('\n=== TESTING COMPLETELY EMPTY CONFIG ===');
const emptyResult = optimize(originalCode, JSON.stringify({ treeShake: {} }));
console.log('Empty config size:', emptyResult.length, 'bytes');
console.log('Empty config reduction:', originalCode.length - emptyResult.length, 'bytes');

console.log('\n=== FINAL ANALYSIS ===');
console.log('Original:', originalCode.length, 'bytes');
console.log('All exports false:', result.length, 'bytes'); 
console.log('+ Webpack tree-shake:', webpackOnly.length, 'bytes');
console.log('Empty config only:', emptyResult.length, 'bytes');

if (webpackOnly.length <= emptyResult.length) {
    console.log('✅ Maximum optimization achieved: All exports false + webpack ≤ empty config');
} else {
    console.log('⚠️ Empty config is more optimal than all exports false + webpack');
}