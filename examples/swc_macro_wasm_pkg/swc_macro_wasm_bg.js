let wasm;
export function __wbg_set_wasm(val) {
	wasm = val;
}

const lTextDecoder =
	typeof TextDecoder === "undefined"
		? (0, module.require)("util").TextDecoder
		: TextDecoder;

let cachedTextDecoder = new lTextDecoder("utf-8", {
	ignoreBOM: true,
	fatal: true
});

cachedTextDecoder.decode();

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
	if (
		cachedUint8ArrayMemory0 === null ||
		cachedUint8ArrayMemory0.byteLength === 0
	) {
		cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
	}
	return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
	ptr = ptr >>> 0;
	return cachedTextDecoder.decode(
		getUint8ArrayMemory0().subarray(ptr, ptr + len)
	);
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder =
	typeof TextEncoder === "undefined"
		? (0, module.require)("util").TextEncoder
		: TextEncoder;

let cachedTextEncoder = new lTextEncoder("utf-8");

const encodeString =
	typeof cachedTextEncoder.encodeInto === "function"
		? function (arg, view) {
				return cachedTextEncoder.encodeInto(arg, view);
			}
		: function (arg, view) {
				const buf = cachedTextEncoder.encode(arg);
				view.set(buf);
				return {
					read: arg.length,
					written: buf.length
				};
			};

function passStringToWasm0(arg, malloc, realloc) {
	if (realloc === undefined) {
		const buf = cachedTextEncoder.encode(arg);
		const ptr = malloc(buf.length, 1) >>> 0;
		getUint8ArrayMemory0()
			.subarray(ptr, ptr + buf.length)
			.set(buf);
		WASM_VECTOR_LEN = buf.length;
		return ptr;
	}

	let len = arg.length;
	let ptr = malloc(len, 1) >>> 0;

	const mem = getUint8ArrayMemory0();

	let offset = 0;

	for (; offset < len; offset++) {
		const code = arg.charCodeAt(offset);
		if (code > 0x7f) break;
		mem[ptr + offset] = code;
	}

	if (offset !== len) {
		if (offset !== 0) {
			arg = arg.slice(offset);
		}
		ptr = realloc(ptr, len, (len = offset + arg.length * 3), 1) >>> 0;
		const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
		const ret = encodeString(arg, view);

		offset += ret.written;
		ptr = realloc(ptr, len, offset, 1) >>> 0;
	}

	WASM_VECTOR_LEN = offset;
	return ptr;
}
/**
 * @param {string} source
 * @param {string} config
 * @returns {string}
 */
export function optimize(source, config) {
	let deferred3_0;
	let deferred3_1;
	try {
		const ptr0 = passStringToWasm0(
			source,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len0 = WASM_VECTOR_LEN;
		const ptr1 = passStringToWasm0(
			config,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len1 = WASM_VECTOR_LEN;
		const ret = wasm.optimize(ptr0, len0, ptr1, len1);
		deferred3_0 = ret[0];
		deferred3_1 = ret[1];
		return getStringFromWasm0(ret[0], ret[1]);
	} finally {
		wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
	}
}

/**
 * @param {string} content
 * @returns {string}
 */
export function parse_webpack_chunk(content) {
	let deferred2_0;
	let deferred2_1;
	try {
		const ptr0 = passStringToWasm0(
			content,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len0 = WASM_VECTOR_LEN;
		const ret = wasm.parse_webpack_chunk(ptr0, len0);
		deferred2_0 = ret[0];
		deferred2_1 = ret[1];
		return getStringFromWasm0(ret[0], ret[1]);
	} finally {
		wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
	}
}

/**
 * @param {string} content
 * @param {string} module_key
 * @returns {string}
 */
export function get_webpack_module_info(content, module_key) {
	let deferred3_0;
	let deferred3_1;
	try {
		const ptr0 = passStringToWasm0(
			content,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len0 = WASM_VECTOR_LEN;
		const ptr1 = passStringToWasm0(
			module_key,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len1 = WASM_VECTOR_LEN;
		const ret = wasm.get_webpack_module_info(ptr0, len0, ptr1, len1);
		deferred3_0 = ret[0];
		deferred3_1 = ret[1];
		return getStringFromWasm0(ret[0], ret[1]);
	} finally {
		wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
	}
}

/**
 * @param {string} content
 * @returns {string}
 */
export function get_webpack_dependency_graph(content) {
	let deferred2_0;
	let deferred2_1;
	try {
		const ptr0 = passStringToWasm0(
			content,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len0 = WASM_VECTOR_LEN;
		const ret = wasm.get_webpack_dependency_graph(ptr0, len0);
		deferred2_0 = ret[0];
		deferred2_1 = ret[1];
		return getStringFromWasm0(ret[0], ret[1]);
	} finally {
		wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
	}
}

/**
 * @param {string} content
 * @param {string} start_module_id
 * @returns {string}
 */
export function get_webpack_dependency_tree(content, start_module_id) {
	let deferred3_0;
	let deferred3_1;
	try {
		const ptr0 = passStringToWasm0(
			content,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len0 = WASM_VECTOR_LEN;
		const ptr1 = passStringToWasm0(
			start_module_id,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len1 = WASM_VECTOR_LEN;
		const ret = wasm.get_webpack_dependency_tree(ptr0, len0, ptr1, len1);
		deferred3_0 = ret[0];
		deferred3_1 = ret[1];
		return getStringFromWasm0(ret[0], ret[1]);
	} finally {
		wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
	}
}

/**
 * @param {string} source
 * @param {string} config
 * @returns {string}
 */
export function optimize_with_prune_result_json(source, config) {
	let deferred3_0;
	let deferred3_1;
	try {
		const ptr0 = passStringToWasm0(
			source,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len0 = WASM_VECTOR_LEN;
		const ptr1 = passStringToWasm0(
			config,
			wasm.__wbindgen_malloc,
			wasm.__wbindgen_realloc
		);
		const len1 = WASM_VECTOR_LEN;
		const ret = wasm.optimize_with_prune_result_json(ptr0, len0, ptr1, len1);
		deferred3_0 = ret[0];
		deferred3_1 = ret[1];
		return getStringFromWasm0(ret[0], ret[1]);
	} finally {
		wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
	}
}

export function __wbg_log_c222819a41e063d3(arg0) {
	console.log(arg0);
}

export function __wbindgen_init_externref_table() {
	const table = wasm.__wbindgen_export_0;
	const offset = table.grow(4);
	table.set(0, undefined);
	table.set(offset + 0, undefined);
	table.set(offset + 1, null);
	table.set(offset + 2, true);
	table.set(offset + 3, false);
}

export function __wbindgen_string_new(arg0, arg1) {
	const ret = getStringFromWasm0(arg0, arg1);
	return ret;
}

export function __wbindgen_throw(arg0, arg1) {
	throw new Error(getStringFromWasm0(arg0, arg1));
}
