import font from "pkg/font.woff2";

export function test(it) {
  it("should correctly import asset even it's shared", () => {
    expect(font.startsWith("assets/font.woff2")).toBe(true);
  });
}
