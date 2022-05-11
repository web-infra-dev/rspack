// https://stackoverflow.com/questions/68424114/next-js-how-to-fetch-localstorage-data-before-client-side-rendering
// 解决 nextJS 无法获取初始localstorage问题

import { useEffect, useState } from 'react';
import { isSSR } from '@/utils/is';

const getDefaultStorage = (key) => {
  if (!isSSR) {
    return localStorage.getItem(key);
  } else {
    return undefined;
  }
};

function useStorage(
  key: string,
  defaultValue?: string
): [string, (string) => void, () => void] {
  const [storedValue, setStoredValue] = useState(
    getDefaultStorage(key) || defaultValue
  );

  const setStorageValue = (value: string) => {
    if (!isSSR) {
      localStorage.setItem(key, value);
      if (value !== storedValue) {
        setStoredValue(value);
      }
    }
  };

  const removeStorage = () => {
    if (!isSSR) {
      localStorage.removeItem('key');
    }
  };

  useEffect(() => {
    const storageValue = localStorage.getItem(key);
    if (storageValue) {
      setStoredValue(storageValue);
    }
  }, []);

  return [storedValue, setStorageValue, removeStorage];
}

export default useStorage;
