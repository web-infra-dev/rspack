type TargetItem =
	| "web"
	| "webworker"
	| "browserslist"
	| "es3"
	| "es5"
	| "es2015"
	| "es2016"
	| "es2017"
	| "es2018"
	| "es2019"
	| "es2020"
	| "es2021"
	| "es2022";

export type Target = TargetItem | TargetItem[] | false;
export type ResolvedTarget = TargetItem[];

export function resolveTargetOptions(target: Target = "web"): ResolvedTarget {
	if (!target) {
		return [];
	}
	if (!Array.isArray(target)) {
		return [target];
	}

	return target;
}
