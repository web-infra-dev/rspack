import { Card, Typography, Avatar, Space, Grid } from '@arco-design/web-react';
import { IconMore } from '@arco-design/web-react/icon';
import React from 'react';
import useLocale from '@/utils/useLocale';
import locale from './locale';
import styles from './style/index.module.less';

interface StudioProps {
  userInfo: {
    name?: string;
    avatar?: string;
  };
}

export default function Studio(props: StudioProps) {
  const t = useLocale(locale);
  const { userInfo } = props;
  return (
    <Card>
      <Grid.Row>
        <Grid.Col span={16}>
          <Typography.Title
            style={{ marginTop: 0, marginBottom: 16 }}
            heading={6}
          >
            {t['monitor.title.studioPreview']}
          </Typography.Title>
        </Grid.Col>
        <Grid.Col span={8} style={{ textAlign: 'right' }}>
          <IconMore />
        </Grid.Col>
      </Grid.Row>
      <div className={styles['studio-wrapper']}>
        <img
          src="http://p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/c788fc704d32cf3b1136c7d45afc2669.png~tplv-uwbnlip3yd-webp.webp"
          className={styles['studio-preview']}
        />
        <div className={styles['studio-bar']}>
          {userInfo && (
            <div>
              <Space size={12}>
                <Avatar size={24}>
                  <img src={userInfo.avatar} />
                </Avatar>
                <Typography.Text>
                  {userInfo.name}
                  {t['monitor.studioPreview.studio']}
                </Typography.Text>
              </Space>
            </div>
          )}
          <Typography.Text type="secondary">
            3,6000 {t['monitor.studioPreview.watching']}
          </Typography.Text>
        </div>
      </div>
    </Card>
  );
}
