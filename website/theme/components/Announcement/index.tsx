import styles from './index.module.scss';

export function Announcement({
  href,
  message,
  display = true,
}: {
  href: string;
  message: string;
  display?: boolean;
}) {
  if (!display) {
    return null;
  }

  return (
    <div className={styles.bar}>
      <a href={href} className={styles.link}>
        {message}
      </a>
    </div>
  );
}
