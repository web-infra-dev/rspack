#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");

// Format export debug messages for better readability with raw but prettified information
function formatExportMessage(message) {
	// Check if it's an RSPACK_EXPORT_DEBUG message
	if (!message.includes("[RSPACK_EXPORT_DEBUG:")) {
		return `\`\`\`\n${message}\n\`\`\`\n`;
	}

	// Extract the type and data
	const typeMatch = message.match(/\[RSPACK_EXPORT_DEBUG:([^\]]+)\]/);
	const type = typeMatch ? typeMatch[1] : "UNKNOWN";
	
	// Remove the debug prefix for parsing
	const dataStr = message.replace(/\[RSPACK_EXPORT_DEBUG:[^\]]+\]\s*/, "");
	
	let formatted = `**Export Type:** \`${type}\`\n\n`;
	
	try {
		// Parse the data dynamically to handle any future additions
		const parsedData = parseExportData(dataStr);
		
		// Only show enhanced formatting for RSPACK_EXPORT_DEBUG logs to minimize output
		if (Object.keys(parsedData).length > 0) {
			formatted += `**Parsed Fields:**\n`;
			// Format each parsed field dynamically with enhanced detail
			for (const [key, value] of Object.entries(parsedData)) {
				const formattedValue = formatValueDetailed(key, value);
				formatted += `- **${key}:** ${formattedValue}\n`;
			}
			formatted += `\n**Raw Message:**\n\`\`\`\n${message}\n\`\`\`\n`;
		} else {
			// Fallback to raw for unparseable content
			formatted += `**Raw Message:**\n\`\`\`\n${message}\n\`\`\`\n`;
		}
		
	} catch (error) {
		// Show parsing error but still include raw data
		console.warn(`⚠️  Failed to parse export message: ${error.message}`);
		formatted += `\n**Parse Error:** ${error.message}\n`;
		formatted += `**Raw Data:** \`${dataStr}\`\n`;
	}
	
	return formatted;
}

// Dynamically parse export data from debug messages
function parseExportData(dataStr) {
	const result = {};
	const pairs = [];
	let current = "";
	let inString = false;
	let braceLevel = 0;
	let parenLevel = 0;
	
	// More robust parsing to handle nested structures
	for (let i = 0; i < dataStr.length; i++) {
		const char = dataStr[i];
		const prevChar = i > 0 ? dataStr[i-1] : '';
		
		if (char === '"' && prevChar !== '\\') {
			inString = !inString;
		}
		
		if (!inString) {
			if (char === '{' || char === '[') braceLevel++;
			if (char === '}' || char === ']') braceLevel--;
			if (char === '(') parenLevel++;
			if (char === ')') parenLevel--;
			
			// Split on commas only at the top level
			if (char === ',' && braceLevel === 0 && parenLevel === 0) {
				pairs.push(current.trim());
				current = "";
				continue;
			}
		}
		
		current += char;
	}
	
	if (current.trim()) {
		pairs.push(current.trim());
	}
	
	// Parse each key-value pair
	for (const pair of pairs) {
		const colonIndex = pair.indexOf(':');
		if (colonIndex > 0) {
			const key = pair.substring(0, colonIndex).trim();
			const value = pair.substring(colonIndex + 1).trim();
			result[key] = value;
		} else {
			// Handle cases where there might be standalone values
			const trimmed = pair.trim();
			if (trimmed) {
				result[`Field_${Object.keys(result).length + 1}`] = trimmed;
			}
		}
	}
	
	return result;
}

// Enhanced dynamic value formatting with detailed raw information for RSPACK_EXPORT_DEBUG logs
function formatValueDetailed(key, value) {
	// Show both raw and formatted information
	let result = `\`${value}\``;
	
	// Add interpretations for common patterns
	const interpretations = [];
	
	// File paths
	if (value.includes("Identifier") && value.includes("/")) {
		const pathMatch = value.match(/Identifier\(u!"([^"]+)"\)/);
		if (pathMatch) {
			let path = pathMatch[1];
			// Make path relative to project root if possible
			if (path.includes('/examples/basic/')) {
				path = path.replace(/.*\/examples\/basic\//, "./");
			} else if (path.includes('/node_modules/')) {
				const nodeModulesIndex = path.lastIndexOf('/node_modules/');
				path = "node_modules" + path.substring(nodeModulesIndex + 13);
			}
			interpretations.push(`📁 Path: \`${path}\``);
		}
	}
	
	// Runtime specifications with details
	if (value.includes("RuntimeSpec")) {
		const runtimeMatch = value.match(/RuntimeSpec \{ inner: \{([^}]+)\}(?:, key: "([^"]+)")?\}/);
		if (runtimeMatch) {
			const runtimes = runtimeMatch[1].split(',')
				.map(r => r.trim().replace(/u!"([^"]+)"/, '$1'))
				.filter(r => r)
				.join(', ');
			const keyInfo = runtimeMatch[2] ? ` (key: ${runtimeMatch[2]})` : '';
			interpretations.push(`⚙️  Runtime: \`${runtimes}${keyInfo}\``);
		}
	}
	
	// Dependency ranges with context
	if (value.includes("DependencyRange")) {
		const rangeMatch = value.match(/DependencyRange \{ end: (\d+), start: (\d+) \}/);
		if (rangeMatch) {
			const start = parseInt(rangeMatch[2]);
			const end = parseInt(rangeMatch[1]);
			const length = end - start;
			interpretations.push(`📍 Range: characters ${start}-${end} (length: ${length})`);
		}
	}
	
	// Module types
	if (value.includes("ModuleType::")) {
		const typeMatch = value.match(/ModuleType::(\w+)/);
		if (typeMatch) {
			interpretations.push(`🧩 Module Type: \`${typeMatch[1]}\``);
		}
	}
	
	// Arrays with count and content preview
	if (value.match(/^\[.*\]$/)) {
		try {
			const parsed = JSON.parse(value.replace(/'/g, '"'));
			if (Array.isArray(parsed)) {
				interpretations.push(`📋 Array: ${parsed.length} items [${parsed.slice(0, 3).join(', ')}${parsed.length > 3 ? '...' : ''}]`);
			}
		} catch (e) {
			const arrayMatch = value.match(/\[([^\]]+)\]/);
			if (arrayMatch) {
				const items = arrayMatch[1].split(',')
					.map(item => item.trim().replace(/['"]/g, ''))
					.filter(item => item);
				interpretations.push(`📋 Array: ${items.length} items [${items.slice(0, 3).join(', ')}${items.length > 3 ? '...' : ''}]`);
			}
		}
	}
	
	// Boolean values with emoji
	if (value === 'true') interpretations.push(`✅ True`);
	if (value === 'false') interpretations.push(`❌ False`);
	if (value === 'None') interpretations.push(`⭕ None`);
	if (value.startsWith('Some(')) interpretations.push(`📦 Some value present`);
	
	// Numbers with context
	if (/^\d+$/.test(value)) {
		const num = parseInt(value);
		if (num > 1000000) {
			interpretations.push(`🔢 Large number: ${(num / 1000000).toFixed(1)}M`);
		} else if (num > 1000) {
			interpretations.push(`🔢 Number: ${(num / 1000).toFixed(1)}K`);
		}
	}
	
	// ConsumeShared information
	if (value.includes("ConsumeShared")) {
		interpretations.push(`🔗 Module Federation ConsumeShared module`);
	}
	
	// Export names and values
	if (key.toLowerCase().includes('name') && value.includes('"')) {
		const nameMatch = value.match(/"([^"]+)"/);
		if (nameMatch) {
			interpretations.push(`🏷️  Export Name: \`${nameMatch[1]}\``);
		}
	}
	
	// Share keys
	if (key.toLowerCase().includes('share') && value.includes('"')) {
		const shareMatch = value.match(/"([^"]+)"/);
		if (shareMatch) {
			interpretations.push(`🔑 Share Key: \`${shareMatch[1]}\``);
		}
	}
	
	// Combine raw value with interpretations
	if (interpretations.length > 0) {
		result += `\n  ${interpretations.join('\n  ')}`;
	}
	
	return result;
}

// Original simpler formatter (keep for backward compatibility)
function formatValue(key, value) {
	// File paths
	if (value.includes("Identifier") && value.includes("/")) {
		const pathMatch = value.match(/Identifier\(u!"([^"]+)"\)/);
		if (pathMatch) {
			let path = pathMatch[1];
			// Make path relative to project root if possible
			if (path.includes('/examples/basic/')) {
				path = path.replace(/.*\/examples\/basic\//, "./");
			} else if (path.includes('/node_modules/')) {
				const nodeModulesIndex = path.lastIndexOf('/node_modules/');
				path = "node_modules" + path.substring(nodeModulesIndex + 13);
			}
			return `\`${path}\``;
		}
	}
	
	// Runtime specifications
	if (value.includes("RuntimeSpec")) {
		const runtimeMatch = value.match(/RuntimeSpec \{ inner: \{([^}]+)\}(?:, key: "([^"]+)")?\}/);
		if (runtimeMatch) {
			const runtimes = runtimeMatch[1].split(',')
				.map(r => r.trim().replace(/u!"([^"]+)"/, '$1'))
				.filter(r => r)
				.join(', ');
			const keyInfo = runtimeMatch[2] ? ` (key: ${runtimeMatch[2]})` : '';
			return `\`${runtimes}${keyInfo}\``;
		}
	}
	
	// Dependency ranges
	if (value.includes("DependencyRange")) {
		const rangeMatch = value.match(/DependencyRange \{ end: (\d+), start: (\d+) \}/);
		if (rangeMatch) {
			return `\`${rangeMatch[2]}-${rangeMatch[1]}\` (characters)`;
		}
	}
	
	// Arrays (exports, names, etc.)
	if (value.match(/^\[.*\]$/)) {
		try {
			// Try to parse as JSON array
			const parsed = JSON.parse(value.replace(/'/g, '"'));
			if (Array.isArray(parsed)) {
				return `\`${parsed.join(', ')}\``;
			}
		} catch (e) {
			// Manual parsing for non-JSON arrays
			const arrayMatch = value.match(/\[([^\]]+)\]/);
			if (arrayMatch) {
				const items = arrayMatch[1].split(',')
					.map(item => item.trim().replace(/['"]/g, ''))
					.filter(item => item);
				return `\`${items.join(', ')}\``;
			}
		}
	}
	
	// Tuples/complex structures - try to format nicely
	if (value.includes('(') && value.includes(')')) {
		// Try to format tuples and complex structures
		const formatted = value
			.replace(/\(/g, ' (')
			.replace(/,/g, ', ')
			.replace(/\s+/g, ' ')
			.trim();
		return `\`${formatted}\``;
	}
	
	// Boolean and simple values
	if (value === 'true' || value === 'false' || value === 'None' || value === 'Some') {
		return `\`${value}\``;
	}
	
	// Numbers
	if (/^\d+(\.\d+)?$/.test(value)) {
		return `\`${value}\``;
	}
	
	// Default: wrap in backticks and clean up whitespace
	const cleaned = value.replace(/\s+/g, ' ').trim();
	return `\`${cleaned}\``;
}

// Find raw log files in .rspack-profile-* directories
function findRawLogFiles() {
	const profileDirs = fs
		.readdirSync(".")
		.filter(
			dir =>
				dir.startsWith(".rspack-profile-") && fs.statSync(dir).isDirectory()
		)
		.map(dir => path.join(dir));

	const logFiles = [];
	for (const dir of profileDirs) {
		const files = fs
			.readdirSync(dir)
			.filter(file => file.startsWith("rspack_log_") && file.endsWith(".json"))
			.map(file => path.join(dir, file));
		logFiles.push(...files);
	}

	return logFiles.sort().reverse(); // Most recent first
}

// Process NDJSON file and filter DEBUG logs
function processRawLogFile(filePath) {
	console.log(`📖 Processing raw log file: ${filePath}`);

	const fileContent = fs.readFileSync(filePath, "utf8");
	const lines = fileContent
		.trim()
		.split("\n")
		.filter(line => line.trim());

	const debugLogs = [];
	let processedLines = 0;
	let debugCount = 0;

	for (const line of lines) {
		processedLines++;
		try {
			const logEntry = JSON.parse(line);
			if (logEntry.level === "DEBUG") {
				// Only include DEBUG logs that contain RSPACK_EXPORT_DEBUG in the message
				if (logEntry.fields && logEntry.fields.message && 
					logEntry.fields.message.includes("RSPACK_EXPORT_DEBUG")) {
					debugCount++;
					debugLogs.push(logEntry);
				}
			}
		} catch (error) {
			console.warn(
				`⚠️  Skipping malformed JSON line ${processedLines}: ${error.message}`
			);
		}
	}

	console.log(
		`📊 Processed ${processedLines} log lines, found ${debugCount} RSPACK_EXPORT_DEBUG entries`
	);
	return debugLogs;
}

// Extract timestamp from file path
function extractTimestamp(filePath) {
	const match = path.basename(filePath).match(/rspack_log_(.+)\.json/);
	return match ? match[1] : new Date().toISOString().replace(/[:.]/g, "-");
}

// Main execution
console.log("🔍 Looking for raw rspack log files...");

// First try to find pre-filtered debug logs (backward compatibility)
const existingDebugFiles = fs
	.readdirSync(".")
	.filter(
		file => file.startsWith("rspack_debug_logs_") && file.endsWith(".json")
	)
	.sort()
	.reverse();

let debugLogs = [];
let timestamp = "";
const filesToClean = [];

if (existingDebugFiles.length > 0) {
	// Use existing pre-filtered debug logs and apply RSPACK_EXPORT_DEBUG filter
	const debugFile = existingDebugFiles[0];
	timestamp = debugFile.match(/rspack_debug_logs_(.+)\.json/)[1];
	console.log(`📁 Found pre-filtered debug logs: ${debugFile}`);

	const allDebugLogs = JSON.parse(fs.readFileSync(debugFile, "utf8"));
	// Filter for RSPACK_EXPORT_DEBUG messages only
	debugLogs = allDebugLogs.filter(log => 
		log.fields && log.fields.message && 
		log.fields.message.includes("RSPACK_EXPORT_DEBUG")
	);
	console.log(`🔍 Filtered to ${debugLogs.length} RSPACK_EXPORT_DEBUG entries from ${allDebugLogs.length} total debug logs`);
	filesToClean.push(debugFile);
} else {
	// Find and process raw log files
	const rawLogFiles = findRawLogFiles();

	if (rawLogFiles.length === 0) {
		console.error("❌ No rspack log files found");
		console.log(
			"💡 Run ./filter-export-tracing.sh first to generate log files"
		);
		process.exit(1);
	}

	// Process the most recent raw log file
	const rawLogFile = rawLogFiles[0];
	debugLogs = processRawLogFile(rawLogFile);
	timestamp = extractTimestamp(rawLogFile);

	// Add all raw log files and their directories to cleanup list
	for (const file of rawLogFiles) {
		filesToClean.push(file);
		const dir = path.dirname(file);
		if (dir.startsWith(".rspack-profile-")) {
			filesToClean.push(dir);
		}
	}
}

console.log(`📝 Processing ${debugLogs.length} debug log entries...`);

try {
	let markdown = "# Rspack Debug Logs\n\n";
	markdown += `**Generated:** ${new Date().toISOString()}\n`;
	markdown += `**Timestamp:** ${timestamp}\n`;
	markdown += `**Total Debug Entries:** ${debugLogs.length}\n\n`;

	if (debugLogs.length === 0) {
		markdown += "*No debug logs found.*\n";
	} else {
		markdown += "## Debug Log Entries\n\n";

		debugLogs.forEach((log, index) => {
			markdown += `### Entry ${index + 1}\n\n`;

			// Format timestamp
			const logTimestamp = new Date(log.timestamp).toISOString();
			markdown += `**Timestamp:** \`${logTimestamp}\`\n`;
			markdown += `**Level:** \`${log.level}\`\n`;
			markdown += `**Target:** \`${log.target}\`\n`;

			if (log.filename) {
				markdown += `**File:** \`${log.filename}\`\n`;
			}

			// Format fields - handle both message and other fields
			if (log.fields) {
				if (log.fields.message) {
					markdown += "**Message:**\n";
					const formattedMessage = formatExportMessage(log.fields.message);
					markdown += formattedMessage;
				}

				// Show other fields
				const otherFields = Object.keys(log.fields).filter(
					key => key !== "message"
				);
				if (otherFields.length > 0) {
					markdown += "**Other Fields:**\n";
					for (const key of otherFields) {
						const value =
							typeof log.fields[key] === "object"
								? JSON.stringify(log.fields[key], null, 2)
								: log.fields[key];
						markdown += `- **${key}:** \`${value}\`\n`;
					}
				}
			}

			// Format span information if present
			if (log.span) {
				markdown += `**Span:** \`${log.span.name || "N/A"}\`\n`;
				if (log.span.perfetto) {
					if (log.span.perfetto.process_name) {
						markdown += `**Process:** \`${log.span.perfetto.process_name}\`\n`;
					}
					if (log.span.perfetto.track_name) {
						markdown += `**Track:** \`${log.span.perfetto.track_name}\`\n`;
					}
				}
			}

			// Format spans array if present
			if (log.spans && log.spans.length > 0) {
				markdown += "**Span Chain:**\n";
				for (let i = 0; i < log.spans.length; i++) {
					const span = log.spans[i];
					markdown += `${i + 1}. \`${span.name}\`\n`;
				}
			}

			markdown += "\n---\n\n";
		});
	}

	// Write markdown file
	const outputFile = `rspack_debug_logs_${timestamp}.md`;
	fs.writeFileSync(outputFile, markdown);

	console.log(`✅ Generated markdown: ${outputFile}`);
	console.log(`📊 Processed ${debugLogs.length} debug log entries`);

	// Clean up all log files and directories after successful processing
	if (filesToClean.length > 0) {
		console.log("🗑️  Cleaning up log files and directories...");
		for (const item of filesToClean) {
			try {
				const stat = fs.statSync(item);
				if (stat.isDirectory()) {
					fs.rmSync(item, { recursive: true, force: true });
					console.log(`   Deleted directory: ${item}`);
				} else {
					fs.unlinkSync(item);
					console.log(`   Deleted file: ${item}`);
				}
			} catch (deleteError) {
				console.warn(
					`   Warning: Could not delete ${item}: ${deleteError.message}`
				);
			}
		}
	}

	console.log("✨ Processing complete! Only markdown file remains.");
} catch (error) {
	console.error("Error processing debug logs:", error.message);
	process.exit(1);
}
