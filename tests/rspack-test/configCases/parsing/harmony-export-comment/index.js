import { requestEngine } from './share';
import used from './used';

used;

const content = __non_webpack_require__("fs").readFileSync(__filename, 'utf-8');
const pureComment = "/* #__PURE__ */";
it("should keep pure comment of unused export default", () => {
  const methodName = "createRequest";
  expect(content).toContain(`${pureComment}${methodName}`)
});

it("should keep pure comment of used export default", () => {
  const methodName = "function __WEBPACK_DEFAULT_EXPORT__()";
  expect(content).toContain(`${pureComment}${methodName}`)
});