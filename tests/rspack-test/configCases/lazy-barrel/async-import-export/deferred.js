import { evaluations } from "./state";
import defer * as consumer from "./deferred-consumer";

it("should await async dependencies of a deferred import-then-export barrel", () => {
	expect(evaluations).toEqual([]);
	expect(consumer.getName()).toBe("file.txt");
	expect(evaluations).toEqual(["consumer"]);
});
