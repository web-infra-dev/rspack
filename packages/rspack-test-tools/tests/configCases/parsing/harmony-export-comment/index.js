import { requestEngine } from './share';

it("should keep pure comment", () => {
  const content = __non_webpack_require__("fs").readFileSync(__filename, 'utf-8');
  const pureComment = "/* #__PURE__ */";
  const methodName = "createRequest";
  expect(content).toContain(`${pureComment}${methodName}`)
});