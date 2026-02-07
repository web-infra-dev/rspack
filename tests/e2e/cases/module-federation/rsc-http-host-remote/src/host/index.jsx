import { createRoot } from 'react-dom/client';
import { App } from './App';
import { readDefaultProbe } from './DefaultProbe';
import { readSsrProbe } from './SsrProbe';

async function bootstrap() {
  await __webpack_init_sharing__('default');
  await __webpack_init_sharing__('ssr');
  await __webpack_init_sharing__('rsc');

  const hostName = __webpack_require__.federation.initOptions.name;
  const federationShare = __FEDERATION__.__SHARE__[hostName] || {};
  const hasReactInScope = (scope) => {
    if (!scope || typeof scope !== 'object') {
      return false;
    }
    if (scope.react) {
      return true;
    }
    return Object.values(scope).some(
      (value) => value && typeof value === 'object' && value.react,
    );
  };
  window.__RSC_MF_E2E__ = {
    hostName,
    scopes: Object.keys(federationShare),
    hasDefaultReact: hasReactInScope(federationShare.default),
    hasSsrReact: hasReactInScope(federationShare.ssr),
    hasRscReact: hasReactInScope(federationShare.rsc),
    defaultProbe: readDefaultProbe(),
    ssrProbe: readSsrProbe(),
  };

  createRoot(document.getElementById('root')).render(<App />);
}

bootstrap();
