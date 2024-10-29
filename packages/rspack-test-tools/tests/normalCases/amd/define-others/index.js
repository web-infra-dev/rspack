import hello from "./hello";
import luckyNumber from './lucky-number';

it("should be able to define any JS values", function () {
	expect(hello).toBe('hello');
	expect(luckyNumber).toBe(7);
});
