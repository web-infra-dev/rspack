import os from "node:os";

// There is a node.js bug on MacOS which causes closing file watchers to be really slow.
// This limits the number of watchers to mitigate the issue.
// https://github.com/nodejs/node/issues/29949
if (
	os.platform() === "darwin" &&
	process.env.WATCHPACK_WATCHER_LIMIT === undefined
) {
	process.env.WATCHPACK_WATCHER_LIMIT = "20";
}
