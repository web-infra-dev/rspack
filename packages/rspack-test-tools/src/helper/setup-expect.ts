import { normalizeCLR } from "./expect/char";
import { normalizeDiff } from "./expect/diff";
import { normalizeDignostics, normalizeError } from "./expect/error";
import { normalizePlaceholder } from "./expect/placeholder";
import { normalizeStats } from "./expect/rspack";
import { toBeTypeOf } from "./expect/to-be-typeof";
import { toEndWith } from "./expect/to-end-with";
import { toMatchFileSnapshot } from "./expect/to-match-file-snapshot";

expect.extend({
	// CHANGE: new test matcher for `rspack-test-tools`
	// @ts-ignore
	toMatchFileSnapshot,
	toBeTypeOf,
	toEndWith
});

const pipes = [normalizeCLR, normalizePlaceholder];

const serialize = (
	str: string,
	extra: Array<(str: string) => string> = []
): string =>
	[...pipes, ...extra].reduce((res, transform) => transform(res), str);

expect.addSnapshotSerializer({
	test(received) {
		return typeof received === "string";
	},
	print(received) {
		return serialize((received as string).trim());
	}
});

// for diff
expect.addSnapshotSerializer({
	test(received) {
		return received?.constructor?.name === "RspackTestDiff";
	},
	print(received, next) {
		return next(normalizeDiff(received as { value: string }));
	}
});

// for errors
expect.addSnapshotSerializer({
	test(received) {
		return received?.constructor?.name === "RspackStatsDiagnostics";
	},
	print(received, next) {
		return next(
			normalizeDignostics(received as { errors: Error[]; warnings: Error[] })
		);
	}
});

expect.addSnapshotSerializer({
	test(received) {
		return typeof received?.message === "string";
	},
	print(received, next) {
		return next(normalizeError(received as Error));
	}
});

// for stats
expect.addSnapshotSerializer({
	test(received) {
		return received?.constructor?.name === "RspackStats";
	},
	print(received, next) {
		return next(normalizeStats(received as { value: string }));
	}
});
