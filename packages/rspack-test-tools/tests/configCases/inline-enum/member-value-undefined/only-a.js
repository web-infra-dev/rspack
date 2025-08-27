import { E } from "./enum?only-a";

export const test = (it, generated) => {
  it("should inline if only enum member A is used", () => {
    // START:ONLY_A
    expect(E.A).toBe("a");
    // END:ONLY_A
    const block = generated.match(/\/\/ START:ONLY_A([\s\S]*)\/\/ END:ONLY_A/)[1];
    expect(block.includes(`(/* inlined export .E.A */ ("a")).toBe("a")`)).toBe(true);
  })

  it("should remove the module if only enum member A is used", () => {
    const inlinedModuleIds = ["./enum?only-a"];
    inlinedModuleIds.forEach(m => {
      expect(generated.includes(`"${m}": (function`)).toBe(false);
    })
  })
}