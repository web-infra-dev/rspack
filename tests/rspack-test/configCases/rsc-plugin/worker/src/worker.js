import { camelCase } from 'lodash-es';

self.onmessage = event => {
  self.postMessage(camelCase(event.data));
};
