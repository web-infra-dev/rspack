import { rspack, builtinMemFs } from '@rspack/browser';
import { files, config } from './files.js';

builtinMemFs.volume.fromJSON({
  ...files,
});

const promise = new Promise((resolve) => {
  rspack(config, () => {
    const json = builtinMemFs.volume.toJSON();

    const outputDOM = document.createElement('div');
    outputDOM.id = 'output';
    outputDOM.innerHTML = json['/dist/main.js'];
    document.body.appendChild(outputDOM);
    resolve();
  });
});

await promise;
