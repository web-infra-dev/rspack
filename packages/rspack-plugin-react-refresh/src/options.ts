export type PluginOptions = {
	include?: string | RegExp | (string | RegExp)[] | null;
	exclude?: string | RegExp | (string | RegExp)[] | null;
	library?: string;
};

const d = <K extends keyof PluginOptions>(
	object: PluginOptions,
	property: K,
	defaultValue?: PluginOptions[K]
) => {
	if (
		typeof object[property] === "undefined" &&
		typeof defaultValue !== "undefined"
	) {
		object[property] = defaultValue;
	}
	return object[property];
};

export function normalizeOptions(options: PluginOptions) {
	d(options, "exclude", /node_modules/i);
	d(options, "include", /\.([cm]js|[jt]sx?|flow)$/i);
	d(options, "library");
	return options;
}
