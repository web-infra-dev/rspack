import d from "./not-supports-const.js"

let stub = d;

it('should not have TDZ error', () => {
  expect(stub).toBe(undefined);
})

export default 1;
