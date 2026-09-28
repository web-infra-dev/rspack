export const pitch = function (remainingRequest) {
  return `import url from ${JSON.stringify(remainingRequest)}; export default url;`;
};
