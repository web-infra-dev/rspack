module.exports = function () {
  const resources = [
    String.raw`\\?\C:\very\long\resource.js?resource-query#fragment`,
    String.raw`\\.\C:\device\resource.js?resource-query#fragment`,
    String.raw`\\?\UNC\server\share\resource.js?resource-query#fragment`,
    `\\\\?\\C:\\escaped\u200b#name.js?resource-query#fragment`,
  ];
  const parsedResources = resources.map((resource) => {
    this.resource = resource;
    return {
      resourcePath: this.resourcePath,
      resourceQuery: this.resourceQuery,
      resourceFragment: this.resourceFragment,
    };
  });

  return `module.exports = ${JSON.stringify(parsedResources)}`;
};
