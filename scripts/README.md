Scripts here are related to development workflow of Rspack.

## Usages

```bash
node ./scripts/xxxx.js
```

- Some scripts require executing under the folder of workspace root. Don't worries, they would validate the environment while executing.

## Guidance

### Writing scripts with zx

[zx](https://github.com/google/zx) is a wonderful tool for Writing scripts using JavaScript.

There are many [ways](https://github.com/google/zx#documentation) to use zx. The way we recommended is import globals explicitly.

```js
import "zx/globals";
```

This allow us to execute every script(some of them might not use zx) in this folder by using the same way: `node ./scripts/xxxx.js`
