module.exports = () => `module.exports = [${Array.from({ length: 128 }, (_, i) => `require('./value.js?${i}')`).join(',')}];`;
