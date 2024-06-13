type RawPreactOptions = {
	library?: string;
};

type PreactOptions = RawPreactOptions | boolean | undefined;

function resolvePreact(preact: PreactOptions): RawPreactOptions | undefined {
	if (typeof preact === "object") {
		return preact;
	} else if (preact === true) {
		return {};
	} else {
		return undefined;
	}
}

export { resolvePreact };
export type { PreactOptions };
