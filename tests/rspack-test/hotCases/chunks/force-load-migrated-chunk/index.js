import { getV } from "./mod";

module.hot.accept("./mod");

it("should keep updating a force-loaded migrated chunk", async () => {
  expect(getV()).toBe(1);
  await NEXT_HMR();
  expect(await getV()).toBe(1);
  await NEXT_HMR();
  expect(await getV()).toBe(2);
});
