import { useLang } from 'rspress/runtime';
import styles from './ApiMeta.module.scss';

/**
 * The Stability Index is learned from https://nodejs.org/api/documentation.html#stability-index
 */
export enum Stability {
  Deprecated = 'Deprecated', // The feature may emit warnings. Backward compatibility is not guaranteed.
  Removed = 'Removed',
  Experimental = 'Experimental', // The feature is not subject to semantic versioning rules
}

export interface ApiMetaProps {
  addedVersion?: string;
  deprecatedVersion?: string;
  removedVersion?: string;
  stability?: Stability;
}

export function ApiMeta(props: ApiMetaProps) {
  let lang = useLang();
  return (
    <div className={styles.wrapper}>
      {props.addedVersion && (
        <span className={`${styles.tag} ${styles.added}`}>
          <a href={`/${lang}/misc/future`}>Added in v{props.addedVersion}</a>
        </span>
      )}
      {props.deprecatedVersion && (
        <span className={`${styles.tag} ${styles.deprecated}`}>
          <a
            href={`/${lang}/misc/future?deprecatedVersion=${props.deprecatedVersion}`}
          >
            Deprecated in v{props.deprecatedVersion}
          </a>
        </span>
      )}
      {props.removedVersion && (
        <span className={`${styles.tag} ${styles.removed}`}>
          <a
            href={`/${lang}/misc/future?removedVersion=${props.removedVersion}`}
          >
            Removed in v{props.removedVersion}
          </a>
        </span>
      )}
      {props.stability && (
        <div
          className={`${styles.tag} ${props.stability ? styles[props.stability.toLowerCase()] : ''}`}
        >
          Stability: {props.stability}
        </div>
      )}
    </div>
  );
}
