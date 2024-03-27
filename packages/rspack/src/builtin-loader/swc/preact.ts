type PreactOptions = {
	library?: string;
};

function resolvePreact(preact: PreactOptions | undefined): PreactOptions {
	return preact ?? {};
}

export { resolvePreact };
export type { PreactOptions };
