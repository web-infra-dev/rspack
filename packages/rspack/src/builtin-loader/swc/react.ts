type RawReactOptions = {
	runtime?: "automatic" | "classic";
	importSource?: string;
	pragma?: string;
	pragmaFrag?: string;
	throwIfNamespace?: boolean;
	development?: boolean;
	useBuiltins?: boolean;
	useSpread?: boolean;
	refresh?: boolean;
};

function resolveReact(react: ReactOptions): RawReactOptions {
	return react ?? {};
}

type ReactOptions = RawReactOptions | undefined;

export { resolveReact };
export type { ReactOptions };
