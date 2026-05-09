const loader = function (c: string) {
  return c.replace(/NEXT_HMR/g, 'NEXT_HMR.bind(null, module)');
};
export { loader };
export default loader;
