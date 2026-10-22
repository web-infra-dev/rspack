export default function (content) {
  return `module.exports = ${JSON.stringify(content)};`;
};
