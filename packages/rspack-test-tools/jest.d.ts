/// <reference types="jest" />

import { DiffOptions } from "jest-diff";

declare interface FileMatcherOptions {
	diff?: DiffOptions;
}

declare global {
	namespace jest {
		interface Matchers<R, T> {
			toMatchFileSnapshot: (
				filename?: string,
				options?: FileMatcherOptions
			) => void;
		}

		interface Expect {
			toMatchFileSnapshot: (
				filename?: string,
				options?: FileMatcherOptions
			) => void;
		}
	}
}
