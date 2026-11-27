export { "x-y" as value } from "./ext.mjs";
import "./side.mjs";

expect(globalThis.__order).toEqual(['ext', 'side']);
