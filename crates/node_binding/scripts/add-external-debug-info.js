const path = require("node:path");
const { pathToFileURL } = require("node:url");
const {
	existsSync,
	readFileSync,
	writeFileSync,
} = require("node:fs");

// WebAssembly DWARF external file convention:
// https://yurydelendik.github.io/webassembly-dwarf/#external-dwarf-file
const SECTION_NAME = "external_debug_info";
const WASM_MAGIC = Buffer.from([0x00, 0x61, 0x73, 0x6d]);
const WASM_VERSION = Buffer.from([0x01, 0x00, 0x00, 0x00]);

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

function decodeULEB128(buffer, offset, fieldName) {
	let result = 0;
	let shift = 0;
	let cursor = offset;

	for (let i = 0; i < 5; i++) {
		const byte = buffer[cursor];
		if (byte === undefined) {
			throw new Error(`Unexpected EOF while reading ${fieldName}`);
		}

		cursor += 1;
		result += (byte & 0x7f) * 2 ** shift;

		if ((byte & 0x80) === 0) {
			return {
				value: result,
				nextOffset: cursor,
			};
		}

		shift += 7;
	}

	throw new Error(`ULEB128 value for ${fieldName} is too large`);
}

function decodeWasmString(buffer, offset, fieldName) {
	const { value: length, nextOffset } = decodeULEB128(buffer, offset, fieldName);
	const endOffset = nextOffset + length;

	if (endOffset > buffer.length) {
		throw new Error(`Unexpected EOF while reading ${fieldName}`);
	}

	return {
		value: buffer.toString("utf8", nextOffset, endOffset),
		nextOffset: endOffset,
	};
}

function createCustomSection(sectionName, sectionData) {
	const payload = Buffer.concat([encodeWasmString(sectionName), sectionData]);
	return Buffer.concat([
		Buffer.from([0x00]),
		encodeULEB128(payload.length),
		payload,
	]);
}

function stripCustomSection(wasmBuffer, sectionName) {
	if (wasmBuffer.length < 8) {
		throw new Error("Wasm file is too short");
	}

	if (!wasmBuffer.subarray(0, 4).equals(WASM_MAGIC)) {
		throw new Error("Invalid wasm magic header");
	}

	if (!wasmBuffer.subarray(4, 8).equals(WASM_VERSION)) {
		throw new Error("Unsupported wasm version");
	}

	const sections = [];
	let offset = 8;

	while (offset < wasmBuffer.length) {
		const sectionStart = offset;
		const sectionId = wasmBuffer[offset];

		if (sectionId === undefined) {
			throw new Error("Unexpected EOF while reading wasm section id");
		}

		offset += 1;

		const { value: sectionSize, nextOffset: payloadStart } = decodeULEB128(
			wasmBuffer,
			offset,
			"section size"
		);
		const sectionEnd = payloadStart + sectionSize;

		if (sectionEnd > wasmBuffer.length) {
			throw new Error("Unexpected EOF while reading wasm section payload");
		}

		if (sectionId === 0x00) {
			const { value: customSectionName } = decodeWasmString(
				wasmBuffer,
				payloadStart,
				"custom section name"
			);

			if (customSectionName !== sectionName) {
				sections.push(wasmBuffer.subarray(sectionStart, sectionEnd));
			}
		} else {
			sections.push(wasmBuffer.subarray(sectionStart, sectionEnd));
		}

		offset = sectionEnd;
	}

	return Buffer.concat([
		wasmBuffer.subarray(0, 8),
		...sections,
	]);
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

	const wasmBuffer = readFileSync(wasmFilePath);
	const wasmWithoutExternalDebugInfo = stripCustomSection(
		wasmBuffer,
		SECTION_NAME
	);

	// The custom section payload is a UTF-8 URL string. We use a `file://` URL
	// here because the sidecar debug wasm lives on disk next to the runtime wasm.
	const externalDebugInfoSection = createCustomSection(
		SECTION_NAME,
		encodeWasmString(pathToFileURL(debugFilePath).href)
	);

	writeFileSync(
		wasmFilePath,
		Buffer.concat([
			wasmWithoutExternalDebugInfo,
			externalDebugInfoSection,
		])
	);
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
