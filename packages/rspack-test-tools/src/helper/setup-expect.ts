import { toBeTypeOf } from "./expect/to-be-typeof";
import { toEndWith } from "./expect/to-end-with";
import { toMatchFileSnapshot } from "./expect/to-match-file-snapshot";
import { serializers } from "./serializers";

expect.extend({
	// CHANGE: new test matcher for `rspack-test-tools`
	// @ts-ignore
	toMatchFileSnapshot,
	toBeTypeOf,
	toEndWith
});

for (const serializer of serializers) {
	expect.addSnapshotSerializer(serializer);
}
