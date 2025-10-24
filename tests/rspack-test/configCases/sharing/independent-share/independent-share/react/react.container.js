module.exports = {
  get: function () { return Promise.resolve(() => 'react-container'); },
  init: function () { return Promise.resolve(); }
};