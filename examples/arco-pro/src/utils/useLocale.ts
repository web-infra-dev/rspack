import { useContext } from 'react';
import { GlobalContext } from '../context';
import defaultLocale from '../locale';

function useLocale(locale = null) {
  const { lang } = useContext(GlobalContext);

  return (locale || defaultLocale)[lang] || {};
}

export default useLocale;
