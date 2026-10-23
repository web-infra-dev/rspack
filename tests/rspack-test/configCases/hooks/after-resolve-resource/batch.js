import b0 from "./b?batch=0";
import b1 from "./b?batch=1";
import b2 from "./b?batch=2";
import b3 from "./b?batch=3";
import b4 from "./b?batch=4";
import b5 from "./b?batch=5";
import b6 from "./b?batch=6";
import b7 from "./b?batch=7";
import b8 from "./b?batch=8";
import b9 from "./b?batch=9";
import b10 from "./b?batch=10";
import b11 from "./b?batch=11";
import b12 from "./b?batch=12";
import b13 from "./b?batch=13";
import b14 from "./b?batch=14";
import b15 from "./b?batch=15";

it("should preserve concurrent async afterResolve calls when batching", () => {
	expect([
		b0,
		b1,
		b2,
		b3,
		b4,
		b5,
		b6,
		b7,
		b8,
		b9,
		b10,
		b11,
		b12,
		b13,
		b14,
		b15
	]).toEqual(new Array(16).fill("c"));
});
