import binding from '@rspack/binding';

const CORE_VERSION = RSPACK_VERSION;

/**
 * Check if these version matches:
 * `@rspack/core`, Binding version
 */
export const checkVersion = () => {
  if (IS_BROWSER) {
    // Why is IS_BROWSER used here:
    // The Wasm binding and the core js runtime are bundled together in `@rspack/browser`,
    // So the checkVersion is not needed.
    return;
  }

  if (CORE_VERSION === binding.EXPECTED_RSPACK_CORE_VERSION) {
    return null;
  }

  // In canary version, version bump is done after binding is built.
  // And to export `EXPECTED_RSPACK_CORE_VERSION` in binding, it relies on the bumped version of @rspack/core.
  // So we can't check the version of @rspack/core and @rspack/binding in canary version.
  // Here we ignore version check for canary version.
  if (CORE_VERSION.includes('canary')) {
    return null;
  }

  return new Error(
    errorMessage(CORE_VERSION, binding.EXPECTED_RSPACK_CORE_VERSION),
  );
};

const errorMessage = (coreVersion: string, expectedCoreVersion: string) => {
  if (process.env.RSPACK_BINDING) {
    return `Unmatched version @rspack/core@${coreVersion} and binding version.

Help:
	Looks like you are using a custom binding (via environment variable 'RSPACK_BINDING=${process.env.RSPACK_BINDING}').
	The expected version of @rspack/core to the current binding is ${expectedCoreVersion}.
`;
  }

  return `Unmatched version @rspack/core@${coreVersion} and @rspack/binding@${expectedCoreVersion}.

Help:
	Please ensure the version of @rspack/binding and @rspack/core is the same.
	The expected version of @rspack/core to the current binding is ${expectedCoreVersion}.
`;
};
