export type PluginOptions = {
	include?: string | RegExp | (string | RegExp)[] | null;
	exclude?: string | RegExp | (string | RegExp)[] | null;
	library?: string;
	forceEnable?: boolean;
};

const d = <K extends keyof PluginOptions>(
	object: PluginOptions,
	property: K,
	defaultValue?: PluginOptions[K]
) => {
	// TODO: should we also add default for null?
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
	d(options, "forceEnable", false);
	return options;
}
