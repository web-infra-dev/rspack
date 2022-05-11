import React from 'react';
import { Alert, Card, Link, Typography, Tag } from '@arco-design/web-react';
import { IconDoubleRight } from '@arco-design/web-react/icon';
import { useSelector } from 'react-redux';
import useLocale from '@/utils/useLocale';
import locale from './locale';
import CodeBlock from './code-block';
import styles from './style/index.module.less';

export default function Welcome() {
  const t = useLocale(locale);
  const userInfo = useSelector((state: any) => state.userInfo) || {};
  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <Typography.Title heading={5} style={{ marginTop: 0 }}>
          {t['welcome.title.welcome']}
        </Typography.Title>
        <Typography.Text type="secondary">
          {userInfo.name}, {userInfo.email}
        </Typography.Text>
      </div>
      <div>
        <Alert type="success" content={t['welcome.invite']} />
        <Card style={{ marginTop: 20 }} title={t['welcome.usage']}>
          <Typography.Title heading={6} style={{ marginTop: 0 }}>
            1. {t['welcome.step.title.pickup']}
          </Typography.Title>
          <Typography.Text>
            {t['welcome.step.content.pickup']}
            <Tag style={{ marginLeft: 8 }}>
              @arco-design/pro-pages-workplace
            </Tag>
          </Typography.Text>

          <Typography.Title heading={6}>
            2. {t['welcome.step.title.install']}
          </Typography.Title>
          <Typography.Text>{t['welcome.step.content.install']}</Typography.Text>
          <CodeBlock code="arco block use @arco-design/pro-pages-workplace" />

          <Typography.Title heading={6} style={{ marginTop: 0 }}>
            3. {t['welcome.step.title.result']}
          </Typography.Title>
          <Typography.Text>{t['welcome.step.content.result']}</Typography.Text>
        </Card>
        <Card style={{ marginTop: 20 }}>
          <Typography.Text>{t['welcome.title.material']}</Typography.Text>
          <div style={{ marginTop: 8 }}>
            <Link
              target="_blank"
              href="https://arco.design/material?category=arco-design-pro"
            >
              {t['welcome.link.material-pro']} <IconDoubleRight />
            </Link>
          </div>
          <div style={{ marginTop: 8 }}>
            <Link target="_blank" href="https://arco.design/material">
              {t['welcome.link.material-all']} <IconDoubleRight />
            </Link>
          </div>
        </Card>
      </div>
    </div>
  );
}
