/// <reference types="jest" />

import type { DiffOptions } from "jest-diff";

declare interface FileMatcherOptions {
	diff?: DiffOptions;
}

declare global {
	namespace jest {
		interface Matchers<R, T> {
			toMatchFileSnapshotSync: (
				filename?: string,
				options?: FileMatcherOptions
			) => void;
		}

		interface Expect {
			toMatchFileSnapshotSync: (
				filename?: string,
				options?: FileMatcherOptions
			) => void;
		}
	}
}
