// @ts-check
/** @type {import("@rspack/core").Configuration} */
const config = [];

/** @type {import("@rspack/core").Configuration} */
export const config2 = [{}];

/** @type {import("@rspack/core").Configuration} */
export const config3 = [{}, {}];

/** @type {import("@rspack/core").Configuration} */
//@ts-expect-error
export const config4 = [{}, { devtool: true }];

module.exports = config;
