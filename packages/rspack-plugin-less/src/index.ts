import path from "path";

const generateOptions = (options: Less.Options): Less.Options => {
	const defaultConfig = {
		enableSourcemap: false
	};
	return {
		...defaultConfig,
		...options
	};
};

export interface Options {
	implementation?: string;
	lessOptions?: Less.Options;
}

export default async function lessLoader(loaderContext) {
	let meta = "";
	const { lessOptions: options, implementation } =
		loaderContext.getOptions() ?? {};

	try {
		let code = loaderContext.source.getCode();
		const final_options = generateOptions({
			filename: loaderContext.resourcePath,
			...options,
			paths: [
				...(options?.paths || ["node_modules"]),
				path.dirname(loaderContext.resourcePath)
			],
			plugins: []
		});

		// eslint-disable-next-line import/no-dynamic-require, global-require
		let lessImplementation;

		if (typeof implementation === "string") {
			lessImplementation = require(implementation);
		} else {
			lessImplementation = (await import("less")).default;
		}
		const result = await lessImplementation.render(code, final_options);
		const { css } = result;

		return {
			content: css,
			meta: meta ? Buffer.from(JSON.stringify(meta)) : ""
		};
	} catch (error) {
		console.log(loaderContext.resourcePath);
		console.log(error);
		throw error;
	}
}
