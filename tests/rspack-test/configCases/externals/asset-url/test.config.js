'use strict';

module.exports = {
  moduleScope(scope, stats) {
    const link = scope.window.document.createElement('link');
    link.rel = 'stylesheet';
    link.href = `bundle${stats().__index__ ?? 0}.css`;
    scope.window.document.head.appendChild(link);
  },
};
