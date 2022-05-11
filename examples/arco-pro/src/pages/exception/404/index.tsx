import React from 'react';
import { Result, Button } from '@arco-design/web-react';
import locale from './locale';
import useLocale from '@/utils/useLocale';
import styles from './style/index.module.less';

function Exception404() {
  const t = useLocale(locale);

  return (
    <div className={styles.wrapper}>
      <Result
        className={styles.result}
        status="404"
        subTitle={t['exception.result.404.description']}
        extra={[
          <Button key="again" style={{ marginRight: 16 }}>
            {t['exception.result.404.retry']}
          </Button>,
          <Button key="back" type="primary">
            {t['exception.result.404.back']}
          </Button>,
        ]}
      />
    </div>
  );
}

export default Exception404;
