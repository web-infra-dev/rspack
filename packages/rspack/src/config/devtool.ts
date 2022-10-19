export type Devtool =
	| false
	| "cheap-source-map"
	| "cheap-module-source-map"
	| "source-map"
	| "inline-cheap-source-map"
	| "inline-cheap-module-source-map"
	| "inline-source-map"
	| "inline-nosources-cheap-module-source-map"
	| "inline-nosources-source-map"
	| "nosources-cheap-source-map"
	| "nosources-cheap-module-source-map"
	| "nosources-source-map"
	| "hidden-nosources-cheap-source-map"
	| "hidden-nosources-cheap-module-source-map"
	| "hidden-nosources-source-map"
	| "hidden-cheap-source-map"
	| "hidden-cheap-module-source-map"
	| "hidden-source-map"
	| "eval-cheap-source-map"
	| "eval-cheap-module-source-map"
	| "eval-source-map"
	| "eval-nosources-cheap-source-map"
	| "eval-nosources-cheap-module-source-map"
	| "eval-nosources-source-map";

export type ResolvedDevtool = string;

export function resolveDevtoolOptions(
	devtool: Devtool = false
): ResolvedDevtool {
	if (devtool === false) return "";
	return devtool;
}

export function isUseSourceMap(devtool: ResolvedDevtool): boolean {
	return (
		devtool.includes("source-map") &&
		(devtool.includes("module") || !devtool.includes("cheap"))
	);
}
