import { useState } from 'react';
import { useI18n } from '../../i18n';
import { usePageData } from 'rspress/runtime';
import IconCloseCircle from './close';

export function Announcement() {
  const t = useI18n();
  const [disable, setDisable] = useState(
    window.localStorage.getItem('disabled-hire') ?? false
  );
  const { page } = usePageData();
  // Only display in homepage
  if (page.pageType !== 'home' || disable) {
    return null;
  }
  return (
    <div
      className="h-8 flex justify-center items-center bg-gradient-to-r from-green-400 via-yellow-300 to-orange-500"
      style={{
        height: '2rem',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
      }}
    >
      <a
        href="https://webinfra.org/about"
        className="hover:underline text-gray-700 font-bold"
        target="_blank"
        rel="noopener noreferrer "
      >
        {t('recruit')}
      </a>
      <IconCloseCircle
        onClick={() => {
          setDisable(true);
          window.localStorage.setItem('disabled-hire', 'true');
        }}
        style={{
          right: 10,
          color: 'white',
          fontSize: 18,
          position: 'absolute',
          cursor: 'pointer',
        }}
      ></IconCloseCircle>
    </div>
  );
}
