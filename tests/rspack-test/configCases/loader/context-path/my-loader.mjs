import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";

function checkModuleLoaders(context) {
	assert.deepEqual(context._module.loaders.map(loader => loader.loader), [fileURLToPath(import.meta.url)]);
}

export default function (content) {
	checkModuleLoaders(this);
	assert.equal(this.data.checkedModuleLoaders, true);
	return (
		"module.exports = " +
		JSON.stringify({
			resourcePath: this.resourcePath,
			prev: content
		})
	);
};

export async function pitch() {
	checkModuleLoaders(this);
	await new Promise(resolve => setImmediate(resolve));
	checkModuleLoaders(this);
	this.data.checkedModuleLoaders = true;
};
