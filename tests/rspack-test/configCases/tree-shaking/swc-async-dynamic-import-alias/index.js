import { _ as swcAsyncToGenerator } from "@swc/helpers/_/_async_to_generator";

it("should stay conservative when the SWC helper request is aliased", async () => {
	const load = swcAsyncToGenerator(function* () {
		const { default: value } = yield import("./module");
		return value;
	});

	const { namespace, value } = await load();
	expect(value).toBe(42);
	expect(namespace.a).toBe(1);
	expect(namespace.unused).toBe(2);
	expect(namespace.usedExports).toBe(true);
});
