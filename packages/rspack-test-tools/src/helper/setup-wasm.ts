// Skip all snapshot tests for wasm because:
// 1. The wasm error backtrace bypasses emnapi.
// 2. Wasm target is a 32-bit platform, where all hash results are diffeference from the native targets.

function toMatchSnapshot() {
	return { pass: true, message: () => "" };
}

function toMatchInlineSnapshot() {
	return { pass: true, message: () => "" };
}

function toMatchFileSnapshotSync() {
	return { pass: true, message: () => "" };
}

expect.extend({
	toMatchSnapshot,
	toMatchInlineSnapshot,
	toMatchFileSnapshotSync
});
