export function Test(count) {
  return <div>{count}</div>;
}

it("should not throw error", async () => {
  expect(Test(0).props.children).toBe(0);
  await import("./main");
})
