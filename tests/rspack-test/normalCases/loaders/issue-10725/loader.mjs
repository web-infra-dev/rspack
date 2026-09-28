const loaderPath = import.meta.filename;

/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	if (this.query === "?load") {
		return `
import { answer } from "./lib";

export default answer;
`;
	}

	const matchResource = `${this.resourcePath}.js`;
	const loader = `${loaderPath}?load`;
	const remaining = this.remainingRequest;
	const request = JSON.parse(
		this.utils.contextify(this.context, `${matchResource}!=!${loader}!${remaining}`)
	);

	this.async();
	this.loadModule(request, (err, source) => {
		this.callback(err, source);
	});
};
