/**
 * Replaces `@rspack/binding` to code that reads env `RSPACK_BINDING` as the custom binding.
 */
function replaceBinding(root) {
	const binding = root.find(`module.exports = require("@rspack/binding");`);
	const bindingPkg = root.find(
		`module.exports = require("@rspack/binding/package.json");`
	);
	return [
		binding.replace(
			`module.exports = require(process.env.RSPACK_BINDING ? process.env.RSPACK_BINDING : "@rspack/binding");`
		),
		bindingPkg.replace(
			`module.exports = require(process.env.RSPACK_BINDING ? require("node:path").resolve(process.env.RSPACK_BINDING, './package.json') : "@rspack/binding/package.json");`
		)
	];
}

exports.replaceBinding = replaceBinding;
