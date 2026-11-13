const loader = function (c: string) {
  return c.replace(/NEXT_HMR(?:_DEFERRED)?/g, '$&.bind(null, module)');
};
export { loader };
export default loader;
