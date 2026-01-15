export default function () {
  const path = JSON.stringify(this.resourcePath);

  return `
    export * from ${path};
  `;
}
