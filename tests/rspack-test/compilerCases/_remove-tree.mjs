import fs from "node:fs";

// On Linux CI, removing a cache directory right after its compiler closed failed with ENOTEMPTY
// in several portable cache cases. Node retries that error when `maxRetries` is set.
export const retry = { maxRetries: 5, retryDelay: 50 };

export function removeTree(target) {
	fs.rmSync(target, { recursive: true, force: true, ...retry });
}
