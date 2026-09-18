/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	var cb = this.async();
	setTimeout(function () {
		cb(null, source);
	}, 500);
};
