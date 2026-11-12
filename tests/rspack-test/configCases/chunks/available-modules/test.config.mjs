export default {
  // The worker/normal-name collision is a graph-only ownership regression. Its
  // worker entry asset is not an ordinary JSONP chunk that the page can execute.
  findBundle: index => [9, 13].includes(index) ? [] : [`main-${index}.js`],
};
