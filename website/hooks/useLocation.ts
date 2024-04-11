import { useLocation as useNativeLocation } from 'rspress/runtime';

function useLocation() {
  const location = useNativeLocation();
  return Object.assign(location, {
    query: new URLSearchParams(location.search),
  });
}

export { useLocation };
