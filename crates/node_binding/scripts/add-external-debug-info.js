const path = require("node:path");
const os = require("node:os");
const { pathToFileURL } = require("node:url");
const {
	copyFileSync,
	existsSync,
	mkdtempSync,
	rmSync,
	writeFileSync,
} = require("node:fs");
const { spawnSync } = require("node:child_process");

// WebAssembly DWARF external file convention:
// https://yurydelendik.github.io/webassembly-dwarf/#external-dwarf-file
const SECTION_NAME = "external_debug_info";
const LLVM_OBJCOPY_CANDIDATES = [
	process.env.LLVM_OBJCOPY,
	"llvm-objcopy",
	"/opt/homebrew/opt/llvm@16/bin/llvm-objcopy",
].filter(Boolean);

function encodeULEB128(value) {
	if (!Number.isInteger(value) || value < 0) {
		throw new Error(`ULEB128 value must be a non-negative integer, got ${value}`);
	}

	const bytes = [];
	let remaining = value;

	do {
		let byte = remaining & 0x7f;
		remaining >>>= 7;

		if (remaining !== 0) {
			byte |= 0x80;
		}

		bytes.push(byte);
	} while (remaining !== 0);

	return Buffer.from(bytes);
}

function encodeWasmString(value) {
	const content = Buffer.from(value, "utf8");
	return Buffer.concat([encodeULEB128(content.length), content]);
}

function resolveLlvmObjcopy() {
	for (const candidate of LLVM_OBJCOPY_CANDIDATES) {
		const result = spawnSync(candidate, ["--version"], {
			stdio: "ignore",
		});
		if (result.status === 0) {
			return candidate;
		}
	}

	throw new Error(
		[
			"Unable to find llvm-objcopy for wasm external_debug_info.",
			"Set LLVM_OBJCOPY or install llvm-objcopy locally.",
		].join(" ")
	);
}

function addExternalDebugInfo(wasmFile, debugFile) {
	const wasmFilePath = path.resolve(wasmFile);
	const debugFilePath = path.resolve(debugFile);

	if (!existsSync(wasmFilePath)) {
		throw new Error(`Wasm file not found: ${wasmFilePath}`);
	}

	if (!existsSync(debugFilePath)) {
		throw new Error(`Debug wasm file not found: ${debugFilePath}`);
	}

	const llvmObjcopy = resolveLlvmObjcopy();
	const tempDir = mkdtempSync(
		path.join(os.tmpdir(), "rspack-external-debug-info-")
	);
	const payloadFile = path.join(tempDir, `${SECTION_NAME}.bin`);
	const outputFile = path.join(tempDir, path.basename(wasmFilePath));

	try {
		// The custom section payload is a UTF-8 URL string. We use a `file://` URL
		// here because the sidecar debug wasm lives on disk next to the runtime wasm.
		writeFileSync(
			payloadFile,
			encodeWasmString(pathToFileURL(debugFilePath).href)
		);

		const result = spawnSync(
			llvmObjcopy,
			[
				`--remove-section=${SECTION_NAME}`,
				`--add-section=${SECTION_NAME}=${payloadFile}`,
				wasmFilePath,
				outputFile,
			],
			{
				stdio: "inherit",
			}
		);

		if (result.status !== 0) {
			throw new Error(
				`llvm-objcopy failed while adding ${SECTION_NAME} to ${wasmFilePath}`
			);
		}

		copyFileSync(outputFile, wasmFilePath);
	} finally {
		rmSync(tempDir, { recursive: true, force: true });
	}
}

if (require.main === module) {
	const [, , wasmFile, debugFile] = process.argv;

	if (!wasmFile || !debugFile) {
		console.error(
			"Usage: node scripts/add-external-debug-info.js <wasm-file> <debug-wasm-file>"
		);
		process.exit(1);
	}

	try {
		addExternalDebugInfo(wasmFile, debugFile);
	} catch (error) {
		console.error(error);
		process.exit(1);
	}
}

module.exports = {
	addExternalDebugInfo,
};
