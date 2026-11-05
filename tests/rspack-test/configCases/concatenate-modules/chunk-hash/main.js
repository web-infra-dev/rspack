import app from "./app";

it("should run", async function () {
  expect(await app()).toBe(["content1", "content2"][__STATS_I__]);
});
