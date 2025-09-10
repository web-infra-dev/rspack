/**
 * @type {import("@rspack/core").LoaderDefinition}
 */
module.exports = function(content, sourceMap, additionalData) {
	const callback = this.async()
	callback(null, `module.exports = ${
		JSON.stringify(
			Object.fromEntries(
				Object.entries(additionalData).filter(([,v]) => typeof v !== 'function')
			)
		)
	}`, sourceMap, additionalData)
}
