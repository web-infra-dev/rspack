import * as binding from "@rspack/binding";

const CORE_VERSION = RSPACK_VERSION;

/**
 * Check if these version matches:
 * `@rspack/core`, Binding version
 */
export const checkVersion = () => {
	if (CORE_VERSION === binding.EXPECTED_RSPACK_CORE_VERSION) {
		return null;
	}

	return new Error(
		errorMessage(CORE_VERSION, binding.EXPECTED_RSPACK_CORE_VERSION)
	);
};

const errorMessage = (coreVersion: string, expectedCoreVersion: string) => {
	if (process.env.RSPACK_BINDING) {
		return `Unmatched version @rspack/core@${coreVersion} and binding version.

Help:
	Looks like you are using a custom binding (via environment variable 'RSPACK_BINDING=${process.env.RSPACK_BINDING}'). The expected version of @rspack/core to the current binding is ${expectedCoreVersion}.
`;
	}

	return `Unmatched version @rspack/core@${coreVersion} and @rspack/binding@${expectedCoreVersion}.

Help:
	Please ensure the version of @rspack/binding and @rspack/core is the same.
`;
};
