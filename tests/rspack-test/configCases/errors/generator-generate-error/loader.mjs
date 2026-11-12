/** @type {import("@rspack/core").LoaderDefinition<{ message: string }>} */
export default function () {
	const callback = this.async();
	const options = this.getOptions();

	callback(new Error(options.message || 'Message'));
};
