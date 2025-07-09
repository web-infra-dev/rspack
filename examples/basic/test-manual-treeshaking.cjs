#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Read the local optimized chunk (already has exports removed by macro)
const chunkPath = path.join(__dirname, 'dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.local-optimized.js');
const chunkContent = fs.readFileSync(chunkPath, 'utf-8');

console.log('=== MANUAL TREE SHAKING ANALYSIS ===\n');
console.log('Analyzing local optimized chunk...');
console.log('Size:', chunkContent.length, 'bytes');

// Extract all module paths and their dependencies
const moduleRegex = /"(\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+)"\s*:\s*function\([^)]*\)\s*\{([^}]+(?:\{[^}]*\}[^}]*)*)\}/g;
const modules = new Map();

let match;
while ((match = moduleRegex.exec(chunkContent)) !== null) {
  const modulePath = match[1];
  const moduleBody = match[2];
  
  // Extract dependencies
  const deps = [];
  const depRegex = /__webpack_require__\("([^"]+)"\)/g;
  let depMatch;
  while ((depMatch = depRegex.exec(moduleBody)) !== null) {
    deps.push(depMatch[1]);
  }
  
  // Check if module exports anything
  const hasExports = moduleBody.includes('__webpack_require__.d(__webpack_exports__');
  
  modules.set(modulePath, {
    name: modulePath.split('/').pop().replace('.js', ''),
    dependencies: deps,
    hasExports: hasExports,
    body: moduleBody.substring(0, 200) + '...' // Sample
  });
}

console.log('\nTotal modules:', modules.size);

// Build dependency graph
console.log('\n--- BUILDING DEPENDENCY GRAPH ---');
const dependencyGraph = new Map();
const reverseDeps = new Map(); // Who depends on each module

for (const [path, info] of modules) {
  dependencyGraph.set(path, new Set(info.dependencies));
  
  // Build reverse dependencies
  for (const dep of info.dependencies) {
    if (!reverseDeps.has(dep)) {
      reverseDeps.set(dep, new Set());
    }
    reverseDeps.get(dep).add(path);
  }
}

// Find modules with no dependents (top-level exports)
const topLevelModules = [];
for (const [path, info] of modules) {
  const dependents = reverseDeps.get(path) || new Set();
  if (dependents.size === 0 && info.hasExports) {
    topLevelModules.push(path);
  }
}

console.log('Top-level modules (no dependents, have exports):', topLevelModules.length);

// The main lodash module should be the primary top-level module
const mainLodash = [...modules].find(([path]) => path.endsWith('/lodash.js'));
if (mainLodash) {
  console.log('\n--- MAIN LODASH MODULE ---');
  console.log('Path:', mainLodash[0]);
  console.log('Dependencies:', mainLodash[1].dependencies.length);
  console.log('Has exports:', mainLodash[1].hasExports);
}

// Simulate tree shaking: Find all reachable modules from main lodash
console.log('\n--- SIMULATING TREE SHAKING ---');
const reachable = new Set();
const queue = [];

// Start from main lodash module (if it exists)
if (mainLodash) {
  queue.push(mainLodash[0]);
  reachable.add(mainLodash[0]);
}

// BFS to find all reachable modules
while (queue.length > 0) {
  const current = queue.shift();
  const deps = dependencyGraph.get(current) || new Set();
  
  for (const dep of deps) {
    if (!reachable.has(dep)) {
      reachable.add(dep);
      queue.push(dep);
    }
  }
}

console.log('Reachable modules from main lodash:', reachable.size);
console.log('Unreachable modules:', modules.size - reachable.size);

// Show some unreachable modules
const unreachable = [...modules.keys()].filter(path => !reachable.has(path));
if (unreachable.length > 0) {
  console.log('\n--- UNREACHABLE MODULES ---');
  console.log('Count:', unreachable.length);
  console.log('Examples:', unreachable.slice(0, 10).map(path => path.split('/').pop()));
}

// Check specific functions
console.log('\n--- CHECKING SPECIFIC FUNCTIONS ---');
const targetFunctions = ['debounce', 'throttle', 'map', 'filter'];
for (const func of targetFunctions) {
  const found = [...modules].find(([path]) => path.endsWith(`/${func}.js`));
  if (found) {
    const [path, info] = found;
    const isReachable = reachable.has(path);
    console.log(`${func}: ${isReachable ? 'REACHABLE' : 'UNREACHABLE'} (deps: ${info.dependencies.length})`);
  } else {
    console.log(`${func}: NOT FOUND as separate module`);
  }
}

// Analyze why all modules might be considered reachable
console.log('\n=== ANALYSIS ===');
console.log('1. In split chunks, there are no explicit entry points');
console.log('2. The tree shaker cannot determine which exports are actually used');
console.log('3. All modules with exports are considered potentially reachable');
console.log(`4. In this chunk: ${[...modules].filter(([_, info]) => info.hasExports).length} modules have exports`);

// Check if all modules are connected
let connectedCount = 0;
for (const [path] of modules) {
  const hasDependents = reverseDeps.has(path) && reverseDeps.get(path).size > 0;
  const hasDependencies = dependencyGraph.has(path) && dependencyGraph.get(path).size > 0;
  if (hasDependents || hasDependencies) {
    connectedCount++;
  }
}

console.log(`5. Connected modules: ${connectedCount}/${modules.size}`);
console.log('6. This means all modules are part of the dependency graph');

// Write detailed analysis
const analysis = {
  totalModules: modules.size,
  reachableFromMain: reachable.size,
  unreachable: unreachable.length,
  topLevelModules: topLevelModules.map(p => p.split('/').pop()),
  moduleDetails: [...modules].slice(0, 20).map(([path, info]) => ({
    name: info.name,
    dependencies: info.dependencies.length,
    hasExports: info.hasExports,
    reachable: reachable.has(path)
  }))
};

fs.writeFileSync('manual-treeshaking-analysis.json', JSON.stringify(analysis, null, 2));
console.log('\nDetailed analysis written to manual-treeshaking-analysis.json');