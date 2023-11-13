export type PluginOptions = {
	include?: string | RegExp | (string | RegExp)[] | null;
	exclude?: string | RegExp | (string | RegExp)[] | null;
};

const d = (
	object: PluginOptions,
	property: keyof PluginOptions,
	defaultValue: PluginOptions[keyof PluginOptions]
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
	return options;
}
