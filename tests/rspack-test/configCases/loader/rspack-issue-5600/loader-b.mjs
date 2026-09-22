/**
 * @type {import("@rspack/core").LoaderDefinition}
 */
export default function(content, sourceMap, additionalData) {
	const callback = this.async()
	callback(null, `module.exports = ${
		JSON.stringify(
			Object.fromEntries(
				Object.entries(additionalData).filter(([,v]) => typeof v !== 'function')
			)
		)
	}`, sourceMap, additionalData)
}
