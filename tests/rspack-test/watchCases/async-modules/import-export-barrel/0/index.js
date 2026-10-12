const consumer = require("./consumer");

it("should update async status through an unchanged import-then-export barrel", async () => {
	expect(consumer instanceof Promise).toBe(WATCH_STEP === "1");
	const { readValue } = await consumer;
	expect(readValue()).toBe(Number(WATCH_STEP));
});
