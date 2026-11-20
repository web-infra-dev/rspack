import ArrayQueue from "../../packages/rspack/src/util/ArrayQueue.ts";

describe("ArrayQueue", () => {
	it("should iterate items in FIFO order after internal reversal", () => {
		const queue = new ArrayQueue();
		for (let i = 0; i < 16; i++) {
			queue.enqueue(i);
		}

		// Triggers reversal of _list into _listReversed (threshold is 16)
		expect(queue.dequeue()).toBe(0);

		// Appends to _list while _listReversed still holds 1..15
		queue.enqueue(16);
		queue.enqueue(17);

		expect(Array.from(queue)).toEqual([
			1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
		]);
	});
});
