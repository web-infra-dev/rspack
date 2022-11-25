import vm from "vm";

/**
 * compile template to js code.
 */
export interface TemplateCompiler<T = any> {
	compile(
		content: string,
		options?: {
			filename: string;
		} & T
	): Promise<string>;
	options?: T;
}

export const defaultTemplateCompiler: TemplateCompiler = {
	async compile(content, options) {
		const template = (await import("lodash.template")).default(content, {
			interpolate: /<%=([\s\S]+?)%>/g,
			variable: "data",
			...options
		});
		return `function template(templateParams) { with(templateParams) { return (${template.source})(); } }\ntemplate`;
	},
	options: {}
};

/**
 * eval js code to js function or js string.
 */
export async function evaluate(
	compiled: string,
	publicPath: string,
	templateFilename: string
): Promise<string | (() => string | Promise<string>)> {
	if (!compiled) {
		return Promise.reject(
			new Error("The templateCompiler didn't provide a compiled result")
		);
	}
	const vmContext = vm.createContext({
		...global,
		HTML_WEBPACK_PLUGIN: true,
		require: require,
		htmlWebpackPluginPublicPath: publicPath,
		URL: require("url").URL,
		__filename: templateFilename
	});
	const vmScript = new vm.Script(compiled, { filename: templateFilename });
	// Evaluate code and cast to string
	let newSource;
	try {
		newSource = vmScript.runInContext(vmContext);
	} catch (e) {
		return Promise.reject(e);
	}
	return typeof newSource === "string" || typeof newSource === "function"
		? Promise.resolve(newSource)
		: Promise.reject(
				new Error(
					'The compiled template "' +
						templateFilename +
						"\" didn't return html."
				)
		  );
}
