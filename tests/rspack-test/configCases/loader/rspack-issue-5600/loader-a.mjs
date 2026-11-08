/**
 * @type {import("@rspack/core").LoaderDefinition}
 */
export default function(content, sourceMap, additionalData) {
	const callback = this.async()
	callback(null, content, sourceMap, {
		str: 'str',
		num: 1,
		toJSON() {
			throw new Error('unreachable')
		}
	})
}
