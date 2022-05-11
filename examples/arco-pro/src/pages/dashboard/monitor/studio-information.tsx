import { Card, Typography, Form, Input, Button } from '@arco-design/web-react';
import React from 'react';
import useLocale from '@/utils/useLocale';
import locale from './locale';

export default function StudioInformation() {
  const t = useLocale(locale);
  return (
    <Card>
      <Typography.Title style={{ marginTop: 0, marginBottom: 16 }} heading={6}>
        {t['monitor.title.studioInfo']}
      </Typography.Title>
      <Form layout="vertical">
        <Form.Item label={t['monitor.studioInfo.label.studioTitle']} required>
          <Input
            placeholder={`王立群${t['monitor.studioInfo.placeholder.studioTitle']}`}
          />
        </Form.Item>
        <Form.Item
          label={t['monitor.studioInfo.label.onlineNotification']}
          required
        >
          <Input.TextArea />
        </Form.Item>
        <Form.Item
          label={t['monitor.studioInfo.label.studioCategory']}
          required
        >
          <Input.Search />
        </Form.Item>
        <Form.Item
          label={t['monitor.studioInfo.label.studioCategory']}
          required
        >
          <Input.Search />
        </Form.Item>
      </Form>
      <Button type="primary">更新</Button>
    </Card>
  );
}
