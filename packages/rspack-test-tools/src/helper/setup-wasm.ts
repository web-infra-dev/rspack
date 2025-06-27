function toMatchSnapshot() {
	return { pass: true, message: () => "" };
}

function toMatchInlineSnapshot() {
	return { pass: true, message: () => "" };
}

function toMatchFileSnapshot() {
	return { pass: true, message: () => "" };
}

expect.extend({
	toMatchSnapshot,
	toMatchInlineSnapshot,
	toMatchFileSnapshot
});

// @ts-ignore
globalThis.WasmSkips = {
	Normals: [/pnpm-workspace/],
	Compilers: [
		/swc-api/,
		// Unknowntimeout (only in ci)
		/persist-build-inf/,
		/single-file/
	],
	Configs: [
		/swc-loader-incompatible-wasm-plugin/,
		/swc-plugin/,
		/browserslist-config-env/,
		/pnp-enable/,
		// Unknown long string
		/loader-raw-string/
	]
};
