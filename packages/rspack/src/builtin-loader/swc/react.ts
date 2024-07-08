import type { RawReactOptions } from "@rspack/binding";

function resolveReact(react: ReactOptions): RawReactOptions {
	return react ?? {};
}

type ReactOptions = RawReactOptions | undefined;

export { resolveReact };
export type { ReactOptions };
