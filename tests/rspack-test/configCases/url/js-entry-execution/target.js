if (typeof document === 'undefined') {
  self.onmessage = () => self.postMessage('executed in worker');
} else {
  globalThis.NEW_URL_SCRIPT_EXECUTED = true;
}
