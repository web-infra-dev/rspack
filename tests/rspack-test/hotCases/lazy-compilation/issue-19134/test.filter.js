// The externals (`fs`, `path`) are Node builtins resolved by the UMD wrapper's
// CommonJS branch via `require`. Restrict to the node target so they resolve
// deterministically; the reproduced bug is target-independent.
module.exports = function (config) {
	return config.target === "async-node";
};
