import React from 'react';
import { Button, Card, Typography, Space } from '@arco-design/web-react';
import {
  IconArrowRight,
  IconStop,
  IconSwap,
  IconTags,
} from '@arco-design/web-react/icon';
import useLocale from '@/utils/useLocale';
import locale from './locale';

export default function QuickOperation() {
  const t = useLocale(locale);
  return (
    <Card>
      <Typography.Title style={{ marginTop: 0, marginBottom: 16 }} heading={6}>
        {t['monitor.title.quickOperation']}
      </Typography.Title>
      <Space direction="vertical" style={{ width: '100%' }} size={10}>
        <Button long icon={<IconTags />}>
          {t['monitor.quickOperation.changeClarity']}
        </Button>
        <Button long icon={<IconSwap />}>
          {t['monitor.quickOperation.switchStream']}
        </Button>
        <Button long icon={<IconStop />}>
          {t['monitor.quickOperation.removeClarity']}
        </Button>
        <Button long icon={<IconArrowRight />}>
          {t['monitor.quickOperation.pushFlowGasket']}
        </Button>
      </Space>
    </Card>
  );
}
