import { step } from './dynamic-module.js';

it("should watch virtual modules", function () {
	expect(step).toBe(WATCH_STEP);
});
