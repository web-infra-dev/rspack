#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load the real WASM optimizer - NO FALLBACKS
async function loadOptimizer() {
	// Load the WASM module from the correct location
	const wasmPath = path.resolve(
		__dirname,
		"../../swc_macro_wasm_pkg/swc_macro_wasm.js"
	);

	if (!fs.existsSync(wasmPath)) {
		throw new Error(`WASM module not found at ${wasmPath}`);
	}

	console.log("Loading real SWC macro WASM optimizer from:", wasmPath);

	// Load the WASM module
	const module = await import(wasmPath);

	// Initialize the WASM module if needed
	if (module.default && typeof module.default === "function") {
		await module.default();
	}

	// Ensure required functions exist
	if (!module.optimize_with_prune_result_json && !module.optimize) {
		throw new Error("WASM module missing required optimize functions");
	}

	// Return the actual optimizer functions - NO FALLBACKS
	return {
		optimize_js: module.optimize_js,
		optimize_with_prune_result_json: module.optimize_with_prune_result_json,
		optimize: module.optimize
	};
}

/**
 * Read and parse share-usage.json files from dist directories
 */
function readShareUsageFiles() {
	const hostJsonPath = path.resolve(__dirname, "../host/dist/share-usage.json");
	const remoteJsonPath = path.resolve(
		__dirname,
		"../remote/dist/share-usage.json"
	);

	const files = [];

	if (fs.existsSync(hostJsonPath)) {
		const hostData = JSON.parse(fs.readFileSync(hostJsonPath, "utf8"));
		files.push({ name: "host", data: hostData, path: hostJsonPath });
	}

	if (fs.existsSync(remoteJsonPath)) {
		const remoteData = JSON.parse(fs.readFileSync(remoteJsonPath, "utf8"));
		files.push({ name: "remote", data: remoteData, path: remoteJsonPath });
	}

	if (files.length === 0) {
		throw new Error("No share-usage.json files found. Please run build first.");
	}

	return files;
}

/**
 * Merge usage data from multiple apps into combined tree-shake flags
 * The new schema already contains treeShake structure with boolean values
 */
function mergeUsageData(files, targetApp) {
	const mergedTreeShake = {};

	// OR merge export usage across all apps
	files.forEach(({ data }) => {
		if (!data.treeShake) return;
		Object.entries(data.treeShake).forEach(([moduleKey, moduleExports]) => {
			if (!mergedTreeShake[moduleKey]) {
				mergedTreeShake[moduleKey] = {};
			}
			Object.entries(moduleExports).forEach(([exportName, isUsed]) => {
				if (exportName === "chunk_characteristics") return; // skip metadata
				if (mergedTreeShake[moduleKey][exportName] !== true) {
					mergedTreeShake[moduleKey][exportName] = Boolean(isUsed);
				}
			});
		});
	});

	return {
		treeShake: mergedTreeShake,
		metadata: {
			timestamp: new Date().toISOString(),
			apps: files.map(f => f.name),
			modules: Object.keys(mergedTreeShake)
		}
	};
}

/**
 * Find shared library chunk files in dist directories
 */
function findSharedChunks(files) {
	const chunks = [];
	const distDirs = {
		host: path.resolve(__dirname, "../host/dist"),
		remote: path.resolve(__dirname, "../remote/dist")
	};

	files.forEach(({ name: app, data }) => {
		const distDir = distDirs[app];
		if (!distDir || !fs.existsSync(distDir)) return;

		if (!data.treeShake) return;

		Object.entries(data.treeShake).forEach(([library, moduleData]) => {
			const chunkFiles = moduleData?.chunk_characteristics?.chunk_files;
			if (!Array.isArray(chunkFiles)) return;

			chunkFiles.forEach(chunkFile => {
				if (typeof chunkFile !== "string" || !chunkFile.endsWith(".js")) return;

				const fullPath = path.join(distDir, chunkFile);
				if (!fs.existsSync(fullPath)) return;

				// Skip maps, originals, optimized
				if (
					chunkFile.endsWith(".map") ||
					chunkFile.endsWith(".original") ||
					chunkFile.endsWith(".optimized.js")
				)
					return;

				chunks.push({
					path: fullPath,
					mapPath: `${fullPath}.map`,
					app,
					filename: chunkFile,
					library
				});
			});
		});
	});

	return chunks;
}

/**
 * Optimize a shared library chunk using SWC macro with tree-shake flags
 */
async function optimizeChunk(
	chunkPath,
	library,
	treeShakeConfig,
	optimizer,
	chunkCharacteristics
) {
	console.log(`Optimizing chunk: ${path.basename(chunkPath)}`);

	try {
		// Enforce: require chunk characteristics only (entryModules removed)
		const hasChunkCharacteristics = Boolean(
			chunkCharacteristics?.entry_module_id
		);
		if (!hasChunkCharacteristics) {
			console.log(
				`Skipping ${path.basename(chunkPath)} for '${library}' - missing chunk characteristics`
			);
			return null;
		}

		const sourceCode = fs.readFileSync(chunkPath, "utf8");

		// Create optimization config for the library - include ALL exports (both true and false)
		const libraryKeepFlags = {};
		if (treeShakeConfig[library]) {
			const exports = Object.entries(treeShakeConfig[library]).filter(
				([key]) => key !== "chunk_characteristics"
			);
			exports.forEach(([exportName, shouldKeep]) => {
				// Pass both true and false values so macros can evaluate correctly
				libraryKeepFlags[exportName] = Boolean(shouldKeep);
			});
		}
		// Conservative skip: if there are no export flags at all, do not attempt optimization
		if (Object.keys(libraryKeepFlags).length === 0) {
			console.log(
				`Skipping ${path.basename(chunkPath)} for '${library}' - no export flags defined`
			);
			return null;
		}

		// Build final treeShake config for THIS library only, attach per-app chunk characteristics
		const treeShakeLib = { ...libraryKeepFlags };
		if (chunkCharacteristics) {
			treeShakeLib.chunk_characteristics = chunkCharacteristics;
		}

		const config = { treeShake: { [library]: treeShakeLib } };

		const configJson = JSON.stringify(config);
		const keptExports = Object.entries(libraryKeepFlags)
			.filter(([, keep]) => keep)
			.map(([name]) => name);
		const removedExports = Object.entries(libraryKeepFlags)
			.filter(([, keep]) => !keep)
			.map(([name]) => name);
		console.log(
			`Tree-shake config for ${library}:`,
			keptExports.length,
			"exports to keep,",
			removedExports.length,
			"to remove"
		);
		if (keptExports.length > 0) {
			console.log("Exports to keep:", keptExports.join(", "));
		}

		// Run SWC macro optimization with prune info output
		if (process.env.DEBUG_OPTIMIZER) {
			console.log(
				`DEBUG: Calling optimizer with config:`,
				JSON.stringify(config, null, 2)
			);
		}
		const jsonStr = optimizer.optimize_with_prune_result_json(
			sourceCode,
			configJson
		);

		let parsed;
		try {
			parsed = JSON.parse(jsonStr);
		} catch (e) {
			throw new Error(`Optimizer did not return valid JSON: ${e.message}`);
		}

		if (parsed?.optimized_source) {
			const optimizedSource = parsed.optimized_source;
			const prune = parsed.prune_result || {};

			// Backup original
			if (!fs.existsSync(`${chunkPath}.original`)) {
				fs.writeFileSync(`${chunkPath}.original`, sourceCode);
			}

			// Write optimized code
			fs.writeFileSync(chunkPath, optimizedSource);

			const removalCount = Array.isArray(prune.removed_modules)
				? prune.removed_modules.length
				: 0;
			const keptCount = Array.isArray(prune.kept_modules)
				? prune.kept_modules.length
				: 0;
			const originalCount =
				typeof prune.original_count === "number" ? prune.original_count : null;

			console.log(`✅ Optimized ${path.basename(chunkPath)}`);
			console.log(`   Original size: ${sourceCode.length} bytes`);
			console.log(`   Optimized size: ${optimizedSource.length} bytes`);
			console.log(
				`   Size reduction: ${(((sourceCode.length - optimizedSource.length) / sourceCode.length) * 100).toFixed(2)}%`
			);
			if (prune.skip_reason) {
				console.log(`   Pruning skipped: ${prune.skip_reason}`);
			} else {
				console.log(
					`   Modules pruned: ${removalCount}${originalCount !== null ? ` (from ${originalCount})` : ""}`
				);
				console.log(`   Modules kept: ${keptCount}`);
			}

			return {
				original_size: sourceCode.length,
				optimized_size: optimizedSource.length,
				reduction:
					((sourceCode.length - optimizedSource.length) / sourceCode.length) *
					100,
				prune
			};
		} else if (parsed?.error) {
			console.log(
				`ℹ️  No optimization applied for ${path.basename(chunkPath)}: ${parsed.error}`
			);
			return null;
		} else {
			console.log(
				`ℹ️  No optimization applied for ${path.basename(chunkPath)} (unexpected response shape)`
			);
			return null;
		}
	} catch (error) {
		console.error(
			`❌ Failed to optimize ${path.basename(chunkPath)}:`,
			error.message
		);
		return null;
	}
}

/**
 * Main optimization function
 * @param {Object} options - Configuration options
 * @param {boolean} options.generateReport - Whether to generate detailed pruning report
 */
async function main(options = {}) {
	const { generateReport = false } = options;
	console.log("🚀 Starting Module Federation chunk optimization...\n");

	try {
		// Load SWC macro optimizer
		console.log("Loading SWC macro optimizer...");
		const optimizer = await loadOptimizer();
		console.log("✅ SWC macro optimizer loaded\n");

		// Read share usage files
		console.log("Reading share-usage.json files...");
		const files = readShareUsageFiles();
		console.log(
			`✅ Found ${files.length} share-usage files: ${files.map(f => f.name).join(", ")}\n`
		);

		// Merge export usage across apps, but NOT chunk characteristics
		console.log("Merging export usage across apps (flags only)...");
		const mergedFlags = mergeUsageData(files).treeShake;
		console.log(
			`✅ Prepared merged export flags for modules: ${Object.keys(mergedFlags).join(", ")}\n`
		);

		// Find shared library chunks
		console.log("Finding shared library chunks...");
		const chunks = findSharedChunks(files);

		console.log(`✅ Found ${chunks.length} shared library chunks\n`);

		if (chunks.length === 0) {
			console.log("No shared library chunks found to optimize.");
			return;
		}

		// Do not group libraries; handle each chunk independently

		// Optimize each chunk individually per library (no context from other chunks)
		console.log("Optimizing chunks...");
		const results = [];
		for (const chunk of chunks) {
			const appData = files.find(f => f.name === chunk.app)?.data;
			const chunkCharacteristics =
				appData?.treeShake?.[chunk.library]?.chunk_characteristics;

			// Build single-lib flags from merged export usage across apps,
			// but attach ONLY this app's chunk_characteristics
			const singleLibFlags = {};
			const mergedLibFlags = mergedFlags[chunk.library] || {};
			singleLibFlags[chunk.library] = { ...mergedLibFlags };
			if (chunkCharacteristics) {
				singleLibFlags[chunk.library].chunk_characteristics =
					chunkCharacteristics;
			}

			const result = await optimizeChunk(
				chunk.path,
				chunk.library,
				singleLibFlags,
				optimizer,
				chunkCharacteristics
			);
			if (result) {
				results.push({
					app: chunk.app,
					filename: chunk.filename,
					library: chunk.library,
					...result
				});
			}
		}

		// Summary
		console.log("\n📊 Optimization Summary:");
		console.log("========================");

		if (results.length > 0) {
			const totalOriginal = results.reduce(
				(sum, r) => sum + r.original_size,
				0
			);
			const totalOptimized = results.reduce(
				(sum, r) => sum + r.optimized_size,
				0
			);
			const totalReduction =
				((totalOriginal - totalOptimized) / totalOriginal) * 100;

			// Group results by library for better display
			const resultsByLibrary = results.reduce((acc, result) => {
				if (!acc[result.library]) acc[result.library] = [];
				acc[result.library].push(result);
				return acc;
			}, {});

			Object.entries(resultsByLibrary).forEach(([library, libResults]) => {
				console.log(`\n${library}:`);
				libResults.forEach(result => {
					console.log(`  ${result.app}/${result.filename}:`);
					console.log(
						`    Size reduction: ${result.reduction.toFixed(2)}% (${result.original_size.toLocaleString()} → ${result.optimized_size.toLocaleString()} bytes)`
					);
					if (result.prune) {
						const pr = result.prune;
						if (pr.skip_reason) {
							console.log(`    Pruning skipped: ${pr.skip_reason}`);
						} else {
							const orig =
								typeof pr.original_count === "number"
									? pr.original_count
									: "unknown";
							const kept = Array.isArray(pr.kept_modules)
								? pr.kept_modules.length
								: "unknown";
							const removed = Array.isArray(pr.removed_modules)
								? pr.removed_modules.length
								: "unknown";
							console.log(
								`    Modules kept/pruned: ${kept}/${removed} (original: ${orig})`
							);
						}
					}
				});
			});

			// Calculate overall statistics
			const totalSaved = totalOriginal - totalOptimized;
			console.log(`\n${"=".repeat(60)}`);
			console.log("🎯 OVERALL OPTIMIZATION RESULTS");
			console.log("=".repeat(60));

			console.log(`\n📦 Bundle Size Analysis:`);
			console.log(
				`  Original Total Size: ${(totalOriginal / 1024 / 1024).toFixed(2)} MB (${totalOriginal.toLocaleString()} bytes)`
			);
			console.log(
				`  Optimized Total Size: ${(totalOptimized / 1024 / 1024).toFixed(2)} MB (${totalOptimized.toLocaleString()} bytes)`
			);
			console.log(`  ----------------------------------------`);

			if (totalSaved > 0) {
				console.log(`  ✅ Total Reduction: ${totalReduction.toFixed(2)}%`);
				console.log(
					`  💾 Total Saved: ${(totalSaved / 1024 / 1024).toFixed(2)} MB (${totalSaved.toLocaleString()} bytes)`
				);
			} else {
				console.log(
					`  ⚠️  Total Increase: ${Math.abs(totalReduction).toFixed(2)}%`
				);
				console.log(
					`  📈 Total Added: ${(Math.abs(totalSaved) / 1024 / 1024).toFixed(2)} MB (${Math.abs(totalSaved).toLocaleString()} bytes)`
				);
			}

			// Library statistics
			const libraryStats = Object.entries(resultsByLibrary)
				.map(([library, libResults]) => {
					const libOriginal = libResults.reduce(
						(sum, r) => sum + r.original_size,
						0
					);
					const libOptimized = libResults.reduce(
						(sum, r) => sum + r.optimized_size,
						0
					);
					const libSaved = libOriginal - libOptimized;
					const libPruneStats = libResults.reduce(
						(acc, r) => {
							if (r.prune && !r.prune.skip_reason) {
								acc.totalPruned += Array.isArray(r.prune.removed_modules)
									? r.prune.removed_modules.length
									: 0;
								acc.totalKept += Array.isArray(r.prune.kept_modules)
									? r.prune.kept_modules.length
									: 0;
							}
							return acc;
						},
						{ totalPruned: 0, totalKept: 0 }
					);

					return {
						library,
						saved: libSaved,
						reduction: (libSaved / libOriginal) * 100,
						modulesPruned: libPruneStats.totalPruned,
						modulesKept: libPruneStats.totalKept
					};
				})
				.sort((a, b) => b.saved - a.saved);

			// Show best optimizations
			const successfulOptimizations = libraryStats.filter(s => s.saved > 0);
			if (successfulOptimizations.length > 0) {
				console.log(`\n🏆 Top Optimizations:`);
				successfulOptimizations.slice(0, 5).forEach((stat, i) => {
					const medal = i === 0 ? "🥇" : i === 1 ? "🥈" : i === 2 ? "🥉" : "  ";
					console.log(`  ${medal} ${stat.library}:`);
					console.log(
						`      Saved: ${(stat.saved / 1024 / 1024).toFixed(2)} MB (${stat.reduction.toFixed(1)}%)`
					);
					if (stat.modulesPruned > 0) {
						console.log(
							`      Modules: ${stat.modulesKept} kept, ${stat.modulesPruned} pruned`
						);
					}
				});
			}

			// Show libraries with increases
			const increasedLibraries = libraryStats.filter(s => s.saved < 0);
			if (increasedLibraries.length > 0) {
				console.log(`\n⚠️  Libraries with Size Increases:`);
				increasedLibraries.forEach(stat => {
					console.log(
						`    ${stat.library}: +${(Math.abs(stat.saved) / 1024).toFixed(1)} KB (${Math.abs(stat.reduction).toFixed(1)}%)`
					);
				});
			}

			// Module pruning summary
			const totalModulesPruned = libraryStats.reduce(
				(sum, s) => sum + s.modulesPruned,
				0
			);
			const totalModulesKept = libraryStats.reduce(
				(sum, s) => sum + s.modulesKept,
				0
			);

			if (totalModulesPruned > 0 || totalModulesKept > 0) {
				console.log(`\n📊 Module Pruning Summary:`);
				console.log(
					`  Total Modules Analyzed: ${totalModulesKept + totalModulesPruned}`
				);
				console.log(`  Modules Kept: ${totalModulesKept}`);
				console.log(`  Modules Pruned: ${totalModulesPruned}`);
				if (totalModulesKept + totalModulesPruned > 0) {
					const pruneRate =
						(totalModulesPruned / (totalModulesKept + totalModulesPruned)) *
						100;
					console.log(`  Prune Rate: ${pruneRate.toFixed(1)}%`);
				}
			}

			console.log(`\n${"=".repeat(60)}`);

			// Generate detailed pruning report if requested
			if (generateReport) {
				console.log("\n📝 Generating detailed pruning report...");
				const reportPath = path.resolve(__dirname, "../optimization_report.md");
				let reportContent = "# Module Federation Optimization Report\n\n";
				reportContent += `Generated: ${new Date().toISOString()}\n\n`;
				reportContent += "## Summary\n\n";
				reportContent += `- **Total Original Size:** ${(totalOriginal / 1024 / 1024).toFixed(2)} MB\n`;
				reportContent += `- **Total Optimized Size:** ${(totalOptimized / 1024 / 1024).toFixed(2)} MB\n`;
				reportContent += `- **Total Size Saved:** ${(totalSaved / 1024 / 1024).toFixed(2)} MB (${totalReduction.toFixed(2)}%)\n`;
				reportContent += `- **Total Modules Analyzed:** ${totalModulesKept + totalModulesPruned}\n`;
				reportContent += `- **Total Modules Pruned:** ${totalModulesPruned}\n\n`;

				reportContent += "## Detailed Results by Library\n\n";

				Object.entries(resultsByLibrary).forEach(([library, libResults]) => {
					reportContent += `### ${library}\n\n`;

					libResults.forEach(result => {
						reportContent += `#### ${result.app}/${result.filename}\n\n`;
						reportContent += `- **Original Size:** ${result.original_size.toLocaleString()} bytes\n`;
						reportContent += `- **Optimized Size:** ${result.optimized_size.toLocaleString()} bytes\n`;
						reportContent += `- **Size Reduction:** ${result.reduction.toFixed(2)}%\n`;

						if (result.prune) {
							const pr = result.prune;
							if (pr.skip_reason) {
								reportContent += `- **Status:** Skipped (${pr.skip_reason})\n`;
							} else {
								reportContent += `- **Module Pruning:**\n`;
								reportContent += `  - Original Modules: ${pr.original_count || "unknown"}\n`;
								reportContent += `  - Modules Kept: ${Array.isArray(pr.kept_modules) ? pr.kept_modules.length : "unknown"}\n`;
								reportContent += `  - Modules Removed: ${Array.isArray(pr.removed_modules) ? pr.removed_modules.length : "unknown"}\n`;

								// Add list of removed modules if available
								if (
									Array.isArray(pr.removed_modules) &&
									pr.removed_modules.length > 0
								) {
									reportContent += "\n**Removed Modules:**\n\n";
									const displayLimit = 50; // Increased limit to show more modules
									const modulesToShow = pr.removed_modules.slice(
										0,
										displayLimit
									);
									modulesToShow.forEach(mod => {
										reportContent += `- \`${mod}\`\n`;
									});
									if (pr.removed_modules.length > displayLimit) {
										reportContent += `\n... and ${pr.removed_modules.length - displayLimit} more removed modules\n`;
									}
								} else if (
									Array.isArray(pr.removed_modules) &&
									pr.removed_modules.length === 0
								) {
									reportContent +=
										"\n**Removed Modules:** None (all modules kept)\n";
								}

								// Add list of kept modules summary (not full list due to size)
								if (
									Array.isArray(pr.kept_modules) &&
									pr.kept_modules.length > 0
								) {
									reportContent += `\n**Kept Modules:** ${pr.kept_modules.length} modules retained\n`;
									// Optionally show first few kept modules if needed
									if (process.env.DEBUG_OPTIMIZER) {
										reportContent +=
											"\n**Sample of Kept Modules (first 10):**\n\n";
										pr.kept_modules.slice(0, 10).forEach(mod => {
											reportContent += `- \`${mod}\`\n`;
										});
									}
								}
							}
						}
						reportContent += "\n";
					});
				});

				fs.writeFileSync(reportPath, reportContent);
				console.log(`✅ Detailed report saved to: ${reportPath}`);
			}
		} else {
			console.log("No chunks were optimized.");
		}

		console.log("\n✅ Module Federation optimization complete!");
	} catch (error) {
		console.error("❌ Optimization failed:", error.message);
		process.exit(1);
	}
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
	// Parse command line arguments
	const args = process.argv.slice(2);
	const options = {
		generateReport: args.includes("--report") || args.includes("-r")
	};

	if (args.includes("--help") || args.includes("-h")) {
		console.log("Usage: node optimize-shared-chunks.js [options]");
		console.log("Options:");
		console.log("  --report, -r    Generate detailed optimization report");
		console.log("  --help, -h      Show this help message");
		process.exit(0);
	}

	main(options).catch(console.error);
}

export { main, mergeUsageData, readShareUsageFiles };
