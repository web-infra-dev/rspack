import a from "./a";
import get from "./b";

var options = { ignoreUnaccepted: true };

it("should ignore unaccepted module updates", async () => {
	expect(a).toBe(2);
	expect(get()).toBe(1);
	await NEXT_HMR(options);
	expect(a).toBe(2);
	await NEXT_HMR(options);
	expect(a).toBe(2);
	expect(get()).toBe(2);
	await NEXT_HMR(options);
	expect(a).toBe(2);
	expect(get()).toBe(3);
});
