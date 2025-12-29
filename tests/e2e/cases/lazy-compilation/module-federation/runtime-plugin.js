module.exports = function () {
  let component;
  return {
    name: 'proxy-remote',
    async errorLoadRemote() {
      if (!component) {
        component = document.createElement('div');
        component.textContent = 'RemoteComponent';
        document.body.appendChild(component);
      }

      return () => component;
    },
  };
};
