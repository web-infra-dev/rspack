import { useState } from 'react';
import { usePageData } from 'rspress/runtime';
import { useLang } from 'rspress/runtime';
import IconCloseCircle from './close';
import styles from './index.module.scss';

const LOCAL_STORAGE_KEY = 'rspack-announcement-closed';
const ANNOUNCEMENT_URL = '/blog/announcing-1-0';

export function Announcement() {
  const [disable, setDisable] = useState(
    window.localStorage.getItem(LOCAL_STORAGE_KEY) ?? false,
  );
  const { page } = usePageData();
  const lang = useLang();

  // Only display in homepage
  if (page.pageType !== 'home' || disable) {
    return null;
  }

  return (
    <div className={`flex justify-center items-center ${styles.bar}`}>
      <a
        href={lang === 'en' ? ANNOUNCEMENT_URL : `/${lang}${ANNOUNCEMENT_URL}`}
        className="hover:underline font-bold"
      >
        {lang === 'en'
          ? 'Rspack v1.0 has been released!'
          : 'Rspack v1.0 正式发布！'}
      </a>
      <IconCloseCircle
        onClick={() => {
          setDisable(true);
          window.localStorage.setItem(LOCAL_STORAGE_KEY, 'true');
        }}
        className={styles.close}
      />
    </div>
  );
}
