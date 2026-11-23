export default function () {
  const options = this.getOptions();
  options.buildCount++;
  return `export default ${JSON.stringify(options.value())};`;
}
