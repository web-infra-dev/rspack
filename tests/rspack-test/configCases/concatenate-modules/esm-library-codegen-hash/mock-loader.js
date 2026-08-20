let generated = false;

module.exports = function () {
  const value = generated ? 'second' : 'first';
  generated = true;
  return `export const value = ${JSON.stringify(value)};`;
};
