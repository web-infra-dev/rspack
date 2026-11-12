/** @type {import("../../../../").LoaderDefinition} */
export default function loader(content) {
  return content.replace(/\/[*/][#]?\s*sourceMappingURL=.+(\*\/)?/g, "");
};
