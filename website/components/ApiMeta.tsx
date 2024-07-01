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
  inline?: boolean;
  specific?: string[];
}

export function ApiMeta(props: ApiMetaProps) {
  let lang = useLang();
  const href = `/${lang}/misc/planning/future`;
  const tagStyle = props.inline ? styles.tagInline : styles.tag;
  const wrapperStyle = props.inline ? styles.wrapperInline : styles.wrapper;
  return (
    <div className={wrapperStyle}>
      {props.addedVersion && (
        <span className={`${tagStyle} ${styles.added}`}>
          <a href={href}>Added in v{props.addedVersion}</a>
        </span>
      )}
      {props.deprecatedVersion && (
        <span className={`${tagStyle} ${styles.deprecated}`}>
          <a href={href}>Deprecated in v{props.deprecatedVersion}</a>
        </span>
      )}
      {props.removedVersion && (
        <span className={`${tagStyle} ${styles.removed}`}>
          <a href={href}>Removed in v{props.removedVersion}</a>
        </span>
      )}
      {props.stability && (
        <div
          className={`${tagStyle} ${
            props.stability ? styles[props.stability.toLowerCase()] : ''
          }`}
        >
          Stability: {props.stability}
        </div>
      )}
      {props.specific && (
        <span className={`${tagStyle} ${styles.specific}`}>
          <a href={href}>{props.specific.join('/')} specific</a>
        </span>
      )}
    </div>
  );
}
