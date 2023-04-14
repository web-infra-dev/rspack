The scripts here are related to development workflow of Rspack.

## Usages

```bash
node ./scripts/xxxx.js
```

- Some scripts require executing under the folder of workspace root. Don't worries, they would validate the environment while executing.

## Guidance

### Writting scripts with zx

[zx](https://github.com/google/zx) is wonderful tool for writting script using JavaScript.

There are many [ways](https://github.com/google/zx#documentation) to use zx. The way I recommended is import globals explicitly.

```js
import "zx/globals";
```

This allow us to execute every script(some of them might not use zx) in this folder by using the same way: `node ./scripts/xxxx.js`
