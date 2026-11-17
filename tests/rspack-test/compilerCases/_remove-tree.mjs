import fs from "node:fs";

// Closing a compiler does not wait for the storage's detached cleanup of stale directories, which
// starts by creating `<cache>/_stale` and every missing directory above it. On Linux CI that put
// the tree back while it was being removed, and `rmdir` of a parent failed with ENOTEMPTY. Node's
// `maxRetries` repeats only the `rmdir` that failed, so walk the tree again.
function remove(target, options) {
	for (let attempt = 1; ; attempt++) {
		try {
			fs.rmSync(target, { recursive: true, maxRetries: 5, retryDelay: 50, ...options });
			return;
		} catch (error) {
			if (error.code !== "ENOTEMPTY" || attempt === 5) throw error;
		}
	}
}

export function removeTree(target) {
	remove(target, { force: true });
}

// Fails when the tree does not exist.
export function removeExistingTree(target) {
	remove(target, {});
}
