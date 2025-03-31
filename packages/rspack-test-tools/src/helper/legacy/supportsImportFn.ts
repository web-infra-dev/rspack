// @ts-nocheck
const nodeVersion = process.versions.node.split(".").map(Number);

export function supportsImportFn() {
	// Segmentation fault in vm with --experimental-vm-modules,
	// which has not been resolved in node 16 yet.
	// https://github.com/nodejs/node/issues/35889
	if (nodeVersion[0] > 16) {
		return true;
	}
	return false;
};
