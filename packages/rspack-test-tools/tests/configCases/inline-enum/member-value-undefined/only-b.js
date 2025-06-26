import { E } from "./enum?only-b";

export const test = (it, generated) => {
  it("should not inline if only enum member B is used", () => {
    // START:ONLY_B
    expect(E.B).toBe(void 0);
    // END:ONLY_B
    const block = generated.match(/\/\/ START:ONLY_B([\s\S]*)\/\/ END:ONLY_B/)[1];
    expect(block.includes(`E.B).toBe(void 0)`)).toBe(true);
  })

  it("should keep the module if only enum member B is used", () => {
    const noInlinedModuleIds = ["./enum?only-b"];
    noInlinedModuleIds.forEach(m => {
      expect(generated.includes(`"${m}": (function`)).toBe(false);
    })
  })
}