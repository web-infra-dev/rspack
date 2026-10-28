/** @type {import("@rspack/core").LoaderDefinition<{}, { minimize: boolean, jsfile: boolean }>} */
export default function () {
	return (
		"module.exports = " +
		JSON.stringify({
			minimize: this.minimize,
			jsfile: this.jsfile
		})
	);
};
