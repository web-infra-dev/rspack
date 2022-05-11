import React from 'react';
import {
  Card,
  Typography,
  Tag,
  Space,
  Descriptions,
} from '@arco-design/web-react';
import useLocale from '@/utils/useLocale';
import locale from './locale';

export default function StudioStatus() {
  const t = useLocale(locale);
  const dataStatus = [
    {
      label: (
        <span>
          <Typography.Text style={{ paddingRight: 8 }}>
            {t['monitor.studioStatus.mainstream']}
          </Typography.Text>
          {t['monitor.studioStatus.bitRate']}
        </span>
      ),
      value: '6 Mbps',
    },
    {
      label: t['monitor.studioStatus.frameRate'],
      value: '60',
    },
    {
      label: (
        <span>
          <Typography.Text style={{ paddingRight: 8 }}>
            {t['monitor.studioStatus.hotStandby']}
          </Typography.Text>
          {t['monitor.studioStatus.bitRate']}
        </span>
      ),
      value: '6 Mbps',
    },
    {
      label: t['monitor.studioStatus.frameRate'],
      value: '60',
    },
    {
      label: (
        <span>
          <Typography.Text style={{ paddingRight: 8 }}>
            {t['monitor.studioStatus.coldStandby']}
          </Typography.Text>
          {t['monitor.studioStatus.bitRate']}
        </span>
      ),
      value: '6 Mbps',
    },
    {
      label: t['monitor.studioStatus.frameRate'],
      value: '60',
    },
  ];
  const dataPicture = [
    {
      label: t['monitor.studioStatus.line'],
      value: '热备',
    },
    {
      label: 'CDN',
      value: 'KS',
    },
    {
      label: t['monitor.studioStatus.play'],
      value: 'FLV',
    },
    {
      label: t['monitor.studioStatus.pictureQuality'],
      value: '原画',
    },
  ];

  return (
    <Card>
      <Space align="start">
        <Typography.Title
          style={{ marginTop: 0, marginBottom: 16 }}
          heading={6}
        >
          {t['monitor.studioStatus.title.studioStatus']}
        </Typography.Title>
        <Tag color="green">{t['monitor.studioStatus.smooth']}</Tag>
      </Space>
      <Descriptions
        colon=": "
        layout="horizontal"
        data={dataStatus}
        column={2}
      />
      <Typography.Title style={{ marginBottom: 16 }} heading={6}>
        {t['monitor.studioStatus.title.pictureInfo']}
      </Typography.Title>
      <Descriptions
        colon=": "
        layout="horizontal"
        data={dataPicture}
        column={2}
      />
    </Card>
  );
}
