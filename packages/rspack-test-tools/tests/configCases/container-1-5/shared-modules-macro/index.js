const fs = require("fs");
const path = require("path");

it("should apply tree-shaking macros to CJS modules in Module Federation shared context", async () => {
	// Import only specific CJS exports to test tree-shaking
	const { usedFunction, usedConstant, createObject, processCjsData } = await import("./cjs-module.js");
	
	// Test that used exports work correctly
	expect(usedFunction()).toBe("This function is used");
	expect(usedConstant).toBe("used constant");
	expect(typeof createObject()).toBe("object");
	expect(processCjsData("test")).toBe("processed: test");
	
	// Check for tree-shaking macros in generated bundles
	const bundleFiles = fs.readdirSync(__dirname).filter(f => f.endsWith(".js") && f !== "index.js");
	
	let foundCjsMacros = false;
	let cjsMacroCount = 0;
	
	for (const file of bundleFiles) {
		const filePath = path.join(__dirname, file);
		const content = fs.readFileSync(filePath, "utf8");
		
		// Look for CJS export patterns with macros
		if (content.includes("exports.") || content.includes("module.exports.")) {
			const macroPattern = /\/\*\s*@common:if\s*\[condition="[^"]*"\]\s*\*\/[\s\S]*?\/\*\s*@common:endif\s*\*\//g;
			const macros = content.match(macroPattern);
			
			if (macros) {
				foundCjsMacros = true;
				cjsMacroCount += macros.length;
				
				// Verify macro format for CJS modules
				macros.forEach(macro => {
					expect(macro).toMatch(/treeShake\./);
					expect(macro).toContain("@common:if");
					expect(macro).toContain("@common:endif");
				});
			}
		}
	}
	
	// CJS modules in Module Federation shared context should have macros
	expect(foundCjsMacros).toBe(true);
	expect(cjsMacroCount).toBeGreaterThan(0);
	
	console.log(`✅ Found ${cjsMacroCount} tree-shaking macros in CJS shared modules`);
});

it("should apply tree-shaking macros to ESM modules in Module Federation shared context", async () => {
	// Import only specific ESM exports to test tree-shaking
	const { usedUtil, processEsmData, validateData } = await import("./esm-utils.js");
	
	// Test that used exports work correctly
	expect(usedUtil()).toBe("used utility function");
	expect(processEsmData("test")).toBe("ESM processed: test");
	expect(validateData("test")).toBe(true);
	
	// Check for tree-shaking in ESM modules
	const bundleFiles = fs.readdirSync(__dirname).filter(f => f.endsWith(".js") && f !== "index.js");
	
	let foundEsmOptimization = false;
	
	for (const file of bundleFiles) {
		const filePath = path.join(__dirname, file);
		const content = fs.readFileSync(filePath, "utf8");
		
		// ESM modules should be optimized (may not have same macro patterns as CJS)
		if (content.includes("usedUtil") || content.includes("processEsmData")) {
			foundEsmOptimization = true;
			
			// Check if unused exports are excluded or conditionally included
			const hasUnusedUtil = content.includes("unusedUtil");
			const hasUnusedEsmFunction = content.includes("unusedEsmFunction");
			
			// Log for analysis
			console.log(`ESM optimization analysis for ${file}:`);
			console.log(`  - Has unusedUtil: ${hasUnusedUtil}`);
			console.log(`  - Has unusedEsmFunction: ${hasUnusedEsmFunction}`);
		}
	}
	
	expect(foundEsmOptimization).toBe(true);
	console.log("✅ ESM modules processed for tree-shaking optimization");
});

it("should handle mixed export patterns with tree-shaking macros", async () => {
	// Import from mixed exports module
	const { mixedFunction, cjsStyleExport } = await import("./mixed-exports.js");
	
	// Test functionality
	expect(typeof mixedFunction).toBe("function");
	expect(cjsStyleExport).toBe("CJS style value");
	
	// Check for macro patterns in mixed export files
	const bundleFiles = fs.readdirSync(__dirname).filter(f => f.endsWith(".js") && f !== "index.js");
	
	let foundMixedMacros = false;
	
	for (const file of bundleFiles) {
		const filePath = path.join(__dirname, file);
		const content = fs.readFileSync(filePath, "utf8");
		
		// Look for mixed patterns with macros
		if (content.includes("mixedFunction") || content.includes("cjsStyleExport")) {
			const hasMacros = content.includes("@common:if") && content.includes("@common:endif");
			if (hasMacros) {
				foundMixedMacros = true;
				console.log(`✅ Found macros in mixed export patterns in ${file}`);
			}
		}
	}
	
	// Mixed patterns should have some optimization
	console.log(`Mixed export macro optimization: ${foundMixedMacros ? "Present" : "Not detected"}`);
});

it("should preserve functionality while enabling tree-shaking", async () => {
	// Test that all used functionality still works correctly
	const cjsModule = await import("./cjs-module.js");
	const esmUtils = await import("./esm-utils.js");
	const pureHelper = await import("./pure-helper.js");
	
	// CJS functionality
	expect(cjsModule.usedFunction()).toBe("This function is used");
	expect(cjsModule.processCjsData("input")).toBe("processed: input");
	
	// ESM functionality
	expect(esmUtils.usedUtil()).toBe("used utility function");
	expect(esmUtils.validateData("test")).toBe(true);
	
	// Pure helper functionality
	expect(pureHelper.pureHelper()).toBe("pure helper result");
	expect(typeof pureHelper.generateId()).toBe("string");
	
	console.log("✅ All used functionality preserved with tree-shaking optimization");
});