// Let's use the webpack_graph crate directly to analyze what the tree shaker sees
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

async function analyzeWithTreeShaker() {
    console.log('=== TREE SHAKER ANALYSIS ===');
    
    // Test on the already optimized lodash chunk
    const optimizedPath = './dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.local-optimized.js';
    const originalPath = './dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js';
    
    console.log('\n=== ANALYZING OPTIMIZED LODASH CHUNK ===');
    console.log('File:', optimizedPath);
    
    // Read the optimized file
    const optimizedCode = fs.readFileSync(optimizedPath, 'utf8');
    console.log('Size:', optimizedCode.length, 'bytes');
    
    // Count webpack require calls
    const requireCalls = optimizedCode.match(/__webpack_require__\(/g) || [];
    console.log('Total __webpack_require__ calls:', requireCalls.length);
    
    // Count PURE annotations
    const pureAnnotations = optimizedCode.match(/\/\* #__PURE__ \*\//g) || [];
    console.log('PURE annotations:', pureAnnotations.length);
    
    // Look for webpack modules structure
    const moduleMatches = optimizedCode.match(/"[^"]+"\s*:\s*function/g) || [];
    console.log('Webpack modules found:', moduleMatches.length);
    
    if (moduleMatches.length > 0) {
        console.log('Module IDs:', moduleMatches.map(m => m.match(/"([^"]+)"/)[1]).slice(0, 10).join(', '), '...');
    }
    
    // Look for entry points
    const entryPoints = optimizedCode.match(/__webpack_require__\s*\(\s*\/\*.*?\*\/\s*"([^"]+)"/g) || [];
    console.log('Entry point calls:', entryPoints.length);
    
    if (entryPoints.length > 0) {
        console.log('Entry points:', entryPoints.map(e => e.match(/"([^"]+)"/)[1]).slice(0, 5).join(', '), '...');
    }
    
    console.log('\n=== COMPARISON WITH ORIGINAL ===');
    const originalCode = fs.readFileSync(originalPath, 'utf8');
    const originalModules = originalCode.match(/"[^"]+"\s*:\s*function/g) || [];
    const originalRequires = originalCode.match(/__webpack_require__\(/g) || [];
    const originalPure = originalCode.match(/\/\* #__PURE__ \*\//g) || [];
    
    console.log('Original modules:', originalModules.length);
    console.log('Original __webpack_require__ calls:', originalRequires.length);
    console.log('Original PURE annotations:', originalPure.length);
    
    console.log('\nReduction:');
    console.log('- Modules removed:', originalModules.length - moduleMatches.length);
    console.log('- Require calls removed:', originalRequires.length - requireCalls.length);
    console.log('- PURE annotations removed:', originalPure.length - pureAnnotations.length);
    
    // Try to identify what got removed
    if (originalModules.length > moduleMatches.length) {
        const originalModuleIds = originalModules.map(m => m.match(/"([^"]+)"/)[1]);
        const optimizedModuleIds = moduleMatches.map(m => m.match(/"([^"]+)"/)[1]);
        const removedModules = originalModuleIds.filter(id => !optimizedModuleIds.includes(id));
        
        console.log('\nRemoved modules:', removedModules.slice(0, 20).join(', '));
        if (removedModules.length > 20) {
            console.log('... and', removedModules.length - 20, 'more');
        }
    }
    
    return {
        optimized: {
            modules: moduleMatches.length,
            requires: requireCalls.length,
            pure: pureAnnotations.length,
            size: optimizedCode.length
        },
        original: {
            modules: originalModules.length,
            requires: originalRequires.length,
            pure: originalPure.length,
            size: originalCode.length
        }
    };
}

async function runRustAnalysis() {
    console.log('\n=== RUNNING RUST WEBPACK_GRAPH ANALYSIS ===');
    
    // Create a simple Rust program to analyze the bundle
    const rustCode = `
use std::fs;
use webpack_graph::{WebpackBundleParser, TreeShaker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let optimized_path = "./dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.local-optimized.js";
    let content = fs::read_to_string(optimized_path)?;
    
    println!("=== RUST WEBPACK_GRAPH ANALYSIS ===");
    println!("File size: {} bytes", content.len());
    
    let parser = WebpackBundleParser::new()?;
    match parser.parse_bundle(&content) {
        Ok(mut graph) => {
            println!("✅ Successfully parsed webpack bundle");
            println!("Total modules: {}", graph.modules.len());
            println!("Entry points: {:?}", graph.entry_points);
            
            // Show first few modules
            let mut module_ids: Vec<_> = graph.modules.keys().collect();
            module_ids.sort();
            println!("Module IDs: {:?}", &module_ids[..module_ids.len().min(10)]);
            
            // Check reachability
            let reachable = graph.get_reachable_modules();
            let unreachable = graph.get_unreachable_modules();
            
            println!("Reachable modules: {}", reachable.len());
            println!("Unreachable modules: {}", unreachable.len());
            
            if !unreachable.is_empty() {
                println!("Unreachable module IDs: {:?}", unreachable);
                
                // Try tree shaking
                let shaken_ids = TreeShaker::new(&mut graph).shake();
                println!("Tree shaker would remove: {:?}", shaken_ids);
            } else {
                println!("✅ All modules are reachable - no dead code found");
            }
        }
        Err(e) => {
            println!("❌ Failed to parse as webpack bundle: {}", e);
            println!("This might not be a standard webpack bundle format");
        }
    }
    
    Ok(())
}
`;
    
    // Write the Rust code to a temporary file
    const rustDir = './temp_analysis';
    if (!fs.existsSync(rustDir)) {
        fs.mkdirSync(rustDir);
    }
    
    // Create Cargo.toml
    const cargoToml = `[package]
name = "bundle_analysis"
version = "0.1.0"
edition = "2021"

[dependencies]
webpack_graph = { path = "../swc_macro_sys/crates/webpack_graph" }
`;
    
    fs.writeFileSync(path.join(rustDir, 'Cargo.toml'), cargoToml);
    
    // Create src directory and main.rs
    const srcDir = path.join(rustDir, 'src');
    if (!fs.existsSync(srcDir)) {
        fs.mkdirSync(srcDir);
    }
    fs.writeFileSync(path.join(srcDir, 'main.rs'), rustCode);
    
    // Run the Rust analysis
    return new Promise((resolve, reject) => {
        const cargo = spawn('cargo', ['run'], { 
            cwd: rustDir,
            stdio: 'inherit'
        });
        
        cargo.on('close', (code) => {
            if (code === 0) {
                resolve();
            } else {
                reject(new Error(`Cargo process exited with code ${code}`));
            }
        });
        
        cargo.on('error', (err) => {
            reject(err);
        });
    });
}

async function main() {
    try {
        const analysis = await analyzeWithTreeShaker();
        await runRustAnalysis();
        
        console.log('\n=== SUMMARY ===');
        console.log('The tree shaker analysis will show us:');
        console.log('1. Whether webpack_graph can parse the optimized bundle');
        console.log('2. How many modules it detects');
        console.log('3. Which modules are reachable vs unreachable');
        console.log('4. Whether our iterative tree-shaking missed anything');
        
    } catch (error) {
        console.error('Analysis failed:', error.message);
    }
}

main();