// The external (`util`) is a Node builtin resolved by the UMD wrapper's
// CommonJS branch via `require`. Restrict to the node target so it resolves
// deterministically; the reproduced bug is target-independent.
module.exports = function (config) {
	return config.target === "async-node";
};
