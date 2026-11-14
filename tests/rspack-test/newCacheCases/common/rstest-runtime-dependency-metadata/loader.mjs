export default function (source) {
  this.getOptions().builtModules.push(this.resourcePath);
  return source;
}
