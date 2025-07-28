import { useLocation as useNativeLocation } from '@rspress/core/runtime';

function useLocation() {
  const location = useNativeLocation();
  return Object.assign(location, {
    query: new URLSearchParams(location.search),
  });
}

export { useLocation };
