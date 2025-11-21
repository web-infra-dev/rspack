export const test = (it) => {
  it("should not have 'Invalid URL' error with relative URL in child compilation", () => {
    const url = new URL("./img.png", import.meta.url);
    expect(url.href.endsWith(".png"))
  })
}
