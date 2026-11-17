/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	var err = new Error("Message");
	err.stack = "Stack\n    at LOADER_EXECUTION (/runner.js:1:1)\n    at runLoaders (/runner.js:2:1)";
	//@ts-expect-error hideStack is not a property on normal errors
	err.hideStack = true;
	throw err;
};
