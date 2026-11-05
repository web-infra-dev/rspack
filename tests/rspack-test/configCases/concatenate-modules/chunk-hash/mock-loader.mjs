/** @type {import("@rspack/core").LoaderDefinition<{}, { mockContent: string }>} */
export default function () {
  return `export const m2 = function () { return ${JSON.stringify(this.mockContent)} };`;
};
