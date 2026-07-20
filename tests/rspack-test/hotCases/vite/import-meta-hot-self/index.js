import "./self";
import "./mixed";

it("runs the Vite self callback and preserves hot data", async () => {
	expect(globalThis.__importMetaHotInitial).toEqual({ value: 1, count: 0 });
	expect(globalThis.__mixedSelfEvaluations).toBe(1);
	expect(globalThis.__mixedSelfAccepted).toBeUndefined();
	await NEXT_HMR();
	expect(globalThis.__importMetaHotAccepted).toEqual({ value: 2, count: 1 });
	expect(globalThis.__importMetaHotDataIdentity).toBe(true);
	expect(globalThis.__importMetaHotDataStoredInWebpackData).toBe(true);
	expect(globalThis.__importMetaHotWebpackDataKeys).toEqual(["webpackValue"]);
	expect(globalThis.__mixedSelfEvaluations).toBe(2);
	expect(globalThis.__mixedSelfWebpackError).toBe("mixed self failure");
	expect(globalThis.__mixedSelfAccepted).toBeUndefined();
	await NEXT_HMR();
	expect(globalThis.__mixedSelfEvaluations).toBe(3);
	expect(globalThis.__mixedSelfAccepted).toEqual({ value: 3, calls: 1 });
});
