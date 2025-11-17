import { toBeTypeOf } from "./expect/to-be-typeof";
import { toEndWith } from "./expect/to-end-with";
import { toMatchFileSnapshotSync } from "./expect/to-match-file-snapshot";
import { serializers } from "./serializers";

expect.extend({
	// CHANGE: new test matcher for `rspack-test-tools`
	// @ts-expect-error
	toMatchFileSnapshotSync,
	toBeTypeOf,
	toEndWith
});

for (const serializer of serializers) {
	expect.addSnapshotSerializer(serializer);
}
