/**
 * The following code is modified based on
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/packages/webpack/src/utils/prefresh.js
 *
 * MIT Licensed
 * Author JoviDeCroock
 * Copyright (c) 2021-Present Preact Team
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/LICENSE
 */

const { isComponent, flush } = require("@prefresh/utils");

// eslint-disable-next-line
const getExports = m => m.exports || m.__proto__.exports;

function isSafeExport(key) {
	return (
		key === "__esModule" ||
		key === "__N_SSG" ||
		key === "__N_SSP" ||
		key === "config"
	);
}

function registerExports(moduleExports, moduleId) {
	self["__PREFRESH__"].register(moduleExports, moduleId + " %exports%");
	if (moduleExports == null || typeof moduleExports !== "object") return;

	for (const key in moduleExports) {
		if (isSafeExport(key)) continue;
		const exportValue = moduleExports[key];
		const typeID = moduleId + " %exports% " + key;
		self["__PREFRESH__"].register(exportValue, typeID);
	}
}

const shouldBind = m => {
	let isCitizen = false;
	const moduleExports = getExports(m);

	if (isComponent(moduleExports)) {
		isCitizen = true;
	}

	if (
		moduleExports === undefined ||
		moduleExports === null ||
		typeof moduleExports !== "object"
	) {
		isCitizen = isCitizen || false;
	} else {
		for (const key in moduleExports) {
			if (key === "__esModule") continue;

			const exportValue = moduleExports[key];
			if (isComponent(exportValue)) {
				isCitizen = isCitizen || true;
			}
		}
	}

	return isCitizen;
};

module.exports = Object.freeze({
	getExports,
	shouldBind,
	flush,
	registerExports
});
