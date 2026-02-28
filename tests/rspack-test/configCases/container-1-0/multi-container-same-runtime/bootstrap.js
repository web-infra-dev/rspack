import a from "A"
import b from "B"

export function test(it) {
  it("should not conflict the moduleMap of the contaier entry module", () => {
    expect(a).toBe('a');
    expect(b).toBe('b');
  })
}
