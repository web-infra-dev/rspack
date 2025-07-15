import { aaa, aaaCanMangle } from "./a";
import * as b from "./b"
import { ca, ca_aaaCanMangle, caCanMangle, cb, cb_bbbCanMangle, cbCanMangle } from "./c";

it("__webpack_exports_info__.xxx.canMangle should be correct", () => {
	expect(aaa).toBe("aaa");
	expect(aaaCanMangle).toBe(true);

	const { bbb, bbbCanMangle } = b;
	expect(bbb).toBe("bbb");
	expect(bbbCanMangle).toBe(true);

	expect(caCanMangle).toBe(true);
	expect(cbCanMangle).toBe(true);
});

it("__webpack_exports_info__.xxx.yyy.canMangle should be correct", () => {
	expect(ca.aaa).toBe("aaa");
	expect(ca_aaaCanMangle).toBe(aaaCanMangle);

	expect(cb.bbb).toBe("bbb");
	expect(cb_bbbCanMangle).toBe(b.bbbCanMangle);
});
