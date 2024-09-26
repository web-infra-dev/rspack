import { useState } from 'react';
import IconCloseCircle from './close';
import styles from './index.module.scss';

export function Announcement({
  href,
  message,
  // we don't need localStorageKey for persist to recommend upgrading
  // localStorageKey,
  display = true,
}: {
  href: string;
  message: string;
  // localStorageKey: string;
  display?: boolean;
}) {
  if (!display) {
    return null;
  }
  const [disable, setDisable] = useState(
    // window.localStorage.getItem(localStorageKey) ?? false,
    false,
  );

  if (disable) {
    return null;
  }

  return (
    <div className={styles.bar}>
      <a href={href} className={styles.link}>
        {message}
      </a>
      <IconCloseCircle
        onClick={() => {
          setDisable(true);
          // window.localStorage.setItem(localStorageKey, 'true');
        }}
        className={styles.close}
      />
    </div>
  );
}
