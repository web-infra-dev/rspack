import fs from "node:fs";
import path from "node:path";

const usageFile = "dist/module-export-usage.json";

// Check if the file exists
if (!fs.existsSync(usageFile)) {
	console.error("❌ File not found:", usageFile);
	console.error(
		"Please ensure the build has completed and generated the module export usage file."
	);
	process.exit(1);
}

console.log(`📊 Reading module export usage from: ${usageFile}\n`);

// Read and parse the usage file
const usageData = JSON.parse(fs.readFileSync(usageFile, "utf8"));

// Get current directory for filtering
const currentDir = process.cwd();
const basicDirPattern = /examples\/basic/;

// Find all modules
const allModules = Object.entries(usageData.modules || {})
	.map(([modulePath, data]) => ({
		path: modulePath,
		...data
	}))
	.sort((a, b) => a.path.localeCompare(b.path)); // Sort by path for easier reading

// Separate modules for display - ALWAYS include modules from basic directory regardless of conditions
const modulesToDisplay = allModules.filter(module => {
	const usedExports = module.used_exports || [];
	const providedExports = module.provided_exports || [];

	// ALWAYS include modules from basic directory regardless of any other conditions
	if (module.path.includes("examples/basic/")) {
		return true;
	}

	// For other modules, apply the filtering logic
	// Skip modules with no export information
	if (providedExports.length === 0) return false;

	// Skip modules with no used exports
	if (usedExports.length === 0) return false;

	// Skip single-export leaf nodes where everything is used (likely internal utilities)
	if (providedExports.length === 1 && usedExports.length === 1) return false;

	return true;
});

const completelyUnusedModules = allModules.filter(module => {
	const usedExports = module.used_exports || [];
	const providedExports = module.provided_exports || [];

	// Skip Module Federation ConsumeShared modules - they handle usage differently
	if (module.module_type === "consume-shared-module") {
		return false;
	}

	// Skip fallback modules if there's a corresponding ConsumeShared module that shows usage
	if (isFallbackModuleWithActiveConsumeShared(module, allModules)) {
		return false;
	}

	// Completely unused = has exports but none are used
	return usedExports.length === 0 && providedExports.length > 0;
});

const noInfoModules = allModules.filter(module => {
	const providedExports = module.provided_exports || [];
	// No info modules = modules with no export information
	return providedExports.length === 0;
});

const singleUsedLeafModules = allModules.filter(module => {
	const usedExports = module.used_exports || [];
	const providedExports = module.provided_exports || [];
	// Single used leaf nodes = 1 export, 1 used (fully optimized leaf nodes)
	return providedExports.length === 1 && usedExports.length === 1;
});

// Separate ConsumeShared modules for special analysis
const consumeSharedModules = allModules.filter(module => {
	return module.module_type === "consume-shared-module";
});

// Function to check if a module is a fallback for an active ConsumeShared module
function isFallbackModuleWithActiveConsumeShared(module, allModules) {
	// Check if this module path appears as a fallback in any ConsumeShared module
	for (const otherModule of allModules) {
		if (otherModule.module_type === "consume-shared-module") {
			// Check if this module is referenced as a fallback in the ConsumeShared identifier
			if (otherModule.module_identifier?.includes(module.path)) {
				// Check if the ConsumeShared module shows any usage
				const consumeSharedUsed = otherModule.used_exports || [];
				const consumeSharedUsesNamespace = otherModule.uses_namespace;

				// If the ConsumeShared module has usage, consider its fallback as "used by proxy"
				if (consumeSharedUsed.length > 0 || consumeSharedUsesNamespace) {
					return true;
				}
			}
		}
	}
	return false;
}

// Function to get clean module name
function getModuleName(fullPath) {
	if (fullPath.includes("node_modules")) {
		// For node_modules, show package/file structure
		const parts = fullPath.split("node_modules/").pop().split("/");
		if (parts.length > 2) {
			return `${parts[0]}/${parts[parts.length - 1]}`;
		}
		return parts.join("/");
	}

	// For local files, show relative path from examples/basic
	if (fullPath.includes("examples/basic")) {
		return fullPath.split("examples/basic/").pop();
	}

	// For other paths, show the filename or last parts
	const parts = fullPath.split("/");
	if (parts.length > 1) {
		return parts.slice(-2).join("/");
	}
	return parts[0];
}

const moduleStats = {
	withUsedExports: 0,
	withUnusedExports: 0,
	totalExports: 0,
	totalUsedExports: 0,
	totalUnusedExports: 0,
	completelyUnused: completelyUnusedModules.length,
	noInfo: noInfoModules.length,
	singleUsedLeaf: singleUsedLeafModules.length
};

// Calculate stats for ALL modules (including hidden ones)
for (const module of allModules) {
	const usedExports = module.used_exports || [];
	const providedExports = module.provided_exports || [];
	const unusedExports = providedExports.filter(
		exp => !usedExports.includes(exp)
	);

	if (usedExports.length > 0) moduleStats.withUsedExports++;
	if (unusedExports.length > 0) moduleStats.withUnusedExports++;
	moduleStats.totalExports += providedExports.length;
	moduleStats.totalUsedExports += usedExports.length;
	moduleStats.totalUnusedExports += unusedExports.length;
}

// Prepare data for text report
const analysisResults = {
	timestamp: new Date().toISOString(),
	summary: {
		totalModules: allModules.length,
		displayedModules: modulesToDisplay.length,
		completelyUnusedModules: moduleStats.completelyUnused,
		noInfoModules: moduleStats.noInfo,
		singleUsedLeafModules: moduleStats.singleUsedLeaf,
		modulesWithUsedExports: moduleStats.withUsedExports,
		modulesWithUnusedExports: moduleStats.withUnusedExports,
		totalExports: moduleStats.totalExports,
		totalUsedExports: moduleStats.totalUsedExports,
		totalUnusedExports: moduleStats.totalUnusedExports,
		usageEfficiency: (
			(moduleStats.totalUsedExports / moduleStats.totalExports) *
			100
		).toFixed(1),
		consumeSharedModules: consumeSharedModules.length
	},
	displayedModules: modulesToDisplay.map(module => {
		const usedExports = module.used_exports || [];
		const providedExports = module.provided_exports || [];
		const unusedExports = providedExports.filter(
			exp => !usedExports.includes(exp)
		);

		return {
			name: getModuleName(module.path),
			path: module.path,
			moduleType: module.module_type,
			exports: {
				total: providedExports.length,
				used: usedExports.length,
				unused: unusedExports.length,
				usedList: usedExports,
				unusedList: unusedExports,
				providedList: providedExports
			},
			usageDetails: module.export_usage_details || [],
			potentiallyUnused: module.potentially_unused_exports || [],
			hasUnusedExports: unusedExports.length > 0,
			hasPotentiallyUnusedExports:
				(module.potentially_unused_exports || []).length > 0,
			usageEfficiency:
				providedExports.length > 0
					? ((usedExports.length / providedExports.length) * 100).toFixed(1)
					: "0"
		};
	}),
	allModules: allModules.map(module => {
		const usedExports = module.used_exports || [];
		const providedExports = module.provided_exports || [];
		const unusedExports = providedExports.filter(
			exp => !usedExports.includes(exp)
		);

		return {
			name: getModuleName(module.path),
			path: module.path,
			moduleType: module.module_type,
			exports: {
				total: providedExports.length,
				used: usedExports.length,
				unused: unusedExports.length,
				usedList: usedExports,
				unusedList: unusedExports,
				providedList: providedExports
			},
			usageDetails: module.export_usage_details || [],
			potentiallyUnused: module.potentially_unused_exports || [],
			hasUnusedExports: unusedExports.length > 0,
			hasPotentiallyUnusedExports:
				(module.potentially_unused_exports || []).length > 0,
			isCompletelyUnused:
				usedExports.length === 0 && providedExports.length > 0,
			usageEfficiency:
				providedExports.length > 0
					? ((usedExports.length / providedExports.length) * 100).toFixed(1)
					: "0"
		};
	}),
	completelyUnusedSummary: {
		count: completelyUnusedModules.length,
		totalUnusedExports: completelyUnusedModules.reduce((sum, module) => {
			return sum + (module.provided_exports?.length || 0);
		}, 0),
		examples: completelyUnusedModules.slice(0, 10).map(module => ({
			name: getModuleName(module.path),
			path: module.path,
			unusedExportCount: module.provided_exports?.length || 0
		}))
	},
	noInfoSummary: {
		count: noInfoModules.length,
		examples: noInfoModules.slice(0, 10).map(module => ({
			name: getModuleName(module.path),
			path: module.path
		}))
	},
	singleUsedLeafSummary: {
		count: singleUsedLeafModules.length,
		examples: singleUsedLeafModules.slice(0, 10).map(module => ({
			name: getModuleName(module.path),
			path: module.path
		}))
	},
	consumeSharedSummary: {
		count: consumeSharedModules.length,
		modules: consumeSharedModules.map(module => {
			const usedExports = module.used_exports || [];
			const providedExports = module.provided_exports || [];
			const unusedExports = providedExports.filter(
				exp => !usedExports.includes(exp)
			);
			return {
				shareKey: module.share_key || "unknown",
				name: getModuleName(module.path),
				path: module.path,
				usedExports: usedExports,
				unusedExports: unusedExports,
				providedExports: providedExports,
				usesNamespace: module.uses_namespace,
				usageDetails: module.export_usage_details || [],
				totalExports: providedExports.length,
				usedCount: usedExports.length,
				unusedCount: unusedExports.length
			};
		})
	}
};

// Write readable text report
const textOutputFile = "basic-modules-analysis.txt";
try {
	let textReport = `📊 Module Export Analysis Report (All Modules)
Generated: ${analysisResults.timestamp}

📈 SUMMARY STATISTICS:
  Total modules analyzed: ${analysisResults.summary.totalModules}
  Displayed modules (with optimization potential): ${analysisResults.summary.displayedModules}
  Completely unused modules (hidden): ${analysisResults.summary.completelyUnusedModules}
  No-info modules (hidden): ${analysisResults.summary.noInfoModules}
  Single-used leaf nodes (hidden): ${analysisResults.summary.singleUsedLeafModules}
  ConsumeShared modules: ${analysisResults.summary.consumeSharedModules}
  Modules with used exports: ${analysisResults.summary.modulesWithUsedExports}
  Modules with unused exports: ${analysisResults.summary.modulesWithUnusedExports}
  Total exports: ${analysisResults.summary.totalExports}
  Used exports: ${analysisResults.summary.totalUsedExports}
  Unused exports: ${analysisResults.summary.totalUnusedExports}
  Overall usage efficiency: ${analysisResults.summary.usageEfficiency}%

🚫 COMPLETELY UNUSED MODULES (${analysisResults.completelyUnusedSummary.count} hidden):
  Total unused exports from hidden modules: ${analysisResults.completelyUnusedSummary.totalUnusedExports}
  Examples of completely unused modules:
`;

	for (const [
		index,
		module
	] of analysisResults.completelyUnusedSummary.examples.entries()) {
		textReport += `    ${index + 1}. ${module.name}: ${module.unusedExportCount} unused exports\n`;
	}

	if (analysisResults.completelyUnusedSummary.count > 10) {
		textReport += `    ... and ${analysisResults.completelyUnusedSummary.count - 10} more completely unused modules\n`;
	}

	textReport += `\n📄 NO-INFO MODULES (${analysisResults.noInfoSummary.count} hidden):
  Modules with no export information (entry points, runtime modules):
`;

	for (const [
		index,
		module
	] of analysisResults.noInfoSummary.examples.entries()) {
		textReport += `    ${index + 1}. ${module.name}\n`;
	}

	if (analysisResults.noInfoSummary.count > 10) {
		textReport += `    ... and ${analysisResults.noInfoSummary.count - 10} more no-info modules\n`;
	}

	textReport += `\n🌿 SINGLE-USED LEAF NODES (${analysisResults.singleUsedLeafSummary.count} hidden):
  Fully optimized modules (1 export, 1 used - no optimization needed):
`;

	for (const [
		index,
		module
	] of analysisResults.singleUsedLeafSummary.examples.entries()) {
		textReport += `    ${index + 1}. ${module.name}\n`;
	}

	if (analysisResults.singleUsedLeafSummary.count > 10) {
		textReport += `    ... and ${analysisResults.singleUsedLeafSummary.count - 10} more single-used leaf nodes\n`;
	}

	textReport += `\n🔗 MODULE FEDERATION CONSUMESHARED ANALYSIS (${analysisResults.consumeSharedSummary.count} modules):\n`;

	if (analysisResults.consumeSharedSummary.count > 0) {
		for (const [
			index,
			module
		] of analysisResults.consumeSharedSummary.modules.entries()) {
			textReport += `    ${index + 1}. ${module.shareKey}: ${module.totalExports} total, ${module.usedCount} used, ${module.unusedCount} unused\n`;
			if (module.usedExports.length > 0) {
				textReport += `       ✅ Used: [${module.usedExports.join(", ")}]\n`;
			}
			if (module.unusedExports.length > 0) {
				textReport += `       ❌ Unused: [${module.unusedExports.slice(0, 10).join(", ")}`;
				if (module.unusedExports.length > 10) {
					textReport += ` ... and ${module.unusedExports.length - 10} more`;
				}
				textReport += "]\n";
			}
			if (module.usesNamespace) {
				textReport += "       🌐 Uses namespace\n";
			}
		}
	} else {
		textReport += "    No ConsumeShared modules found\n";
	}

	textReport += "\n🔍 TOP OPTIMIZATION OPPORTUNITIES:\n";

	// Find modules with most unused exports for optimization recommendations
	const optimizationTargets = analysisResults.displayedModules
		.filter(m => m.exports.unused > 0)
		.sort((a, b) => b.exports.unused - a.exports.unused)
		.slice(0, 10);

	if (optimizationTargets.length > 0) {
		textReport += "  Top displayed modules with unused exports:\n";
		for (const [index, module] of optimizationTargets.entries()) {
			textReport += `    ${index + 1}. ${module.name}: ${module.exports.unused} unused (${(100 - Number.parseFloat(module.usageEfficiency)).toFixed(1)}% waste)\n`;
		}
	} else {
		textReport += "  🎉 No displayed modules with unused exports found!\n";
	}

	textReport +=
		"\n📦 DETAILED MODULE ANALYSIS (Modules with Optimization Potential):\n\n";

	// Add detailed module information for displayed modules only
	for (const module of analysisResults.displayedModules) {
		textReport += `Module: ${module.name}\n`;
		textReport += `  Path: ${module.path}\n`;
		if (module.moduleType) {
			textReport += `  Type: ${module.moduleType}\n`;
		}
		textReport += `  Exports: ${module.exports.total} total, ${module.exports.used} used, ${module.exports.unused} unused (${module.usageEfficiency}% efficiency)\n`;

		if (module.exports.usedList.length > 0) {
			textReport += `  ✅ Used: ${module.exports.usedList.slice(0, 10).join(", ")}`;
			if (module.exports.usedList.length > 10) {
				textReport += ` ... and ${module.exports.usedList.length - 10} more`;
			}
			textReport += "\n";
		}

		// Calculate definitively unused vs potentially unused for text report
		const definitelyUnused = module.exports.unusedList.filter(
			exp => !module.potentiallyUnused.includes(exp)
		);
		const onlyPotentiallyUnused = module.potentiallyUnused.filter(
			exp => !module.exports.unusedList.includes(exp)
		);

		if (definitelyUnused.length > 0) {
			textReport += `  ❌ Unused (confirmed): ${definitelyUnused.slice(0, 10).join(", ")}`;
			if (definitelyUnused.length > 10) {
				textReport += ` ... and ${definitelyUnused.length - 10} more`;
			}
			textReport += "\n";
		}

		if (onlyPotentiallyUnused.length > 0) {
			textReport += `  ⚠️  Potentially unused (need analysis): ${onlyPotentiallyUnused.slice(0, 10).join(", ")}`;
			if (onlyPotentiallyUnused.length > 10) {
				textReport += ` ... and ${onlyPotentiallyUnused.length - 10} more`;
			}
			textReport += "\n";
		}

		textReport += "\n";
	}

	textReport += "\n🚀 OPTIMIZATION RECOMMENDATIONS:\n";

	const highWasteModules = analysisResults.allModules.filter(m => {
		if (m.exports.total > 5 && Number.parseFloat(m.usageEfficiency) < 50) {
			// Check if this is a fallback module with active ConsumeShared usage
			const hasActiveConsumeShared =
				analysisResults.consumeSharedSummary.modules.some(csModule => {
					return (
						csModule.path.includes(m.path) &&
						(csModule.usedExports.length > 0 || csModule.usesNamespace)
					);
				});
			return !hasActiveConsumeShared;
		}
		return false;
	});

	if (highWasteModules.length > 0) {
		textReport += "\n1. Consider tree-shaking or replacing these modules:\n";
		textReport +=
			"   Note: Modules with active ConsumeShared usage are excluded from optimization recommendations.\n";
		for (const module of highWasteModules) {
			textReport += `   • ${module.name}: Only ${module.usageEfficiency}% of exports used\n`;
		}
	}

	const largeUnusedModules = analysisResults.allModules.filter(
		m => m.exports.unused > 20
	);
	if (largeUnusedModules.length > 0) {
		textReport += "\n2. Modules with many unused exports (>20):\n";
		for (const module of largeUnusedModules) {
			textReport += `   • ${module.name}: ${module.exports.unused} unused exports\n`;
		}
	}

	const noUsageModules = analysisResults.allModules.filter(m => {
		// Don't include modules that have ConsumeShared wrappers showing usage
		if (m.exports.used === 0 && m.exports.total > 0) {
			// Check if this is a fallback module with active ConsumeShared usage
			const hasActiveConsumeShared =
				analysisResults.consumeSharedSummary.modules.some(csModule => {
					return (
						csModule.path.includes(m.path) &&
						(csModule.usedExports.length > 0 || csModule.usesNamespace)
					);
				});
			return !hasActiveConsumeShared;
		}
		return false;
	});
	if (noUsageModules.length > 0) {
		textReport += "\n3. Modules with no used exports (consider removing):\n";
		textReport +=
			"   Note: Modules with active ConsumeShared usage are excluded from this list.\n";
		for (const module of noUsageModules) {
			textReport += `   • ${module.name}: ${module.exports.total} total exports, none used\n`;
		}
	}

	fs.writeFileSync(textOutputFile, textReport);
} catch (error) {
	process.exit(1);
}
