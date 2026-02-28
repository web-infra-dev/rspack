var s=0;
import { foo as b } from "./a";
b(()=>{s++}),b(()=>{s++}),b(()=>{s++}),b(()=>{
	it("should generate correct code", () => {
		expect(s).toBe(3)
	})
})
