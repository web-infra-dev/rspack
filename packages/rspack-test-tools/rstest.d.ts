/// <reference types="@rstest/core/globals" />

import type { DiffOptions } from "jest-diff";

declare interface FileMatcherOptions {
	diff?: DiffOptions;
}

declare module "@rstest/core" {
	interface Assertion {
		toMatchFileSnapshotSync: (
			filename?: string,
			options?: FileMatcherOptions
		) => void;
	}
}

declare global {
	type Expect = import("@rstest/core").Expect;
	type Describe = import("@rstest/core").Describe;
	type Assertion<T> = import("@rstest/core").Assertion<T>;
}

export {};
