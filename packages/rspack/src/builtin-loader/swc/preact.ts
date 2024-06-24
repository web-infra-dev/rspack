type RawPreactOptions = {
	library?: string;
};

export type PreactOptions = RawPreactOptions | boolean | undefined;

export function resolvePreact(
	preact: PreactOptions
): RawPreactOptions | undefined {
	if (typeof preact === "object") {
		return preact;
	} else if (preact === true) {
		return {};
	} else {
		return undefined;
	}
}
