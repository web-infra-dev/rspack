module.exports = {
  "extensions": {
    "pluginImport": [
      {
        "libraryName": "foo",
        "customName": (name) => {
          if (name === 'PascalCase') {
            return undefined
          } else {
            return `foo/__custom_es__/${name}`
          }
        }
      }
    ]
  }
}
