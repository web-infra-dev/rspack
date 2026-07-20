import { other, val2c, Test } from "./shared";
import { checkedOther } from "./checked-shared";
import { smallCheckedOther } from "./small-checked-shared";

it("should have correct runtime id", () => {
	expect(other).toBe("other");
	expect(val2c).toBe(42);
	expect(Test).toBeTypeOf("function");
	expect(new Test()).toBeInstanceOf(Test);
	expect(checkedOther).toBe("checked-other");
	expect(smallCheckedOther).toBe("small-checked-other");
	expect(__webpack_require__.j).toBe("b-runtime");
});
