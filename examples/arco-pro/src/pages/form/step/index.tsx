import React, { useState } from 'react';
import {
  Steps,
  Form,
  Input,
  Select,
  DatePicker,
  InputTag,
  Button,
  Typography,
  Space,
  Card,
  Switch,
  Result,
} from '@arco-design/web-react';
import useLocale from '@/utils/useLocale';
import locale from './locale';
import styles from './style/index.module.less';

const { Title, Paragraph } = Typography;
function StepForm() {
  const t = useLocale(locale);
  const [current, setCurrent] = useState(1);

  const [form] = Form.useForm();

  const viewForm = () => {
    const values = form.getFields();
    form.setFields(values);
    setCurrent(1);
  };

  const reCreateForm = () => {
    form.resetFields();
    setCurrent(1);
  };

  const toNext = async () => {
    try {
      await form.validate();
      setCurrent(current + 1);
    } catch (_) {}
  };
  return (
    <div className={styles.container}>
      <Card>
        <Title heading={5}>{t['stepForm.desc.basicInfo']}</Title>
        <div className={styles.wrapper}>
          <Steps current={current} lineless>
            <Steps.Step
              title={t['stepForm.title.basicInfo']}
              description={t['stepForm.desc.basicInfo']}
            />
            <Steps.Step
              title={t['stepForm.title.channel']}
              description={t['stepForm.desc.channel']}
            />
            <Steps.Step
              title={t['stepForm.title.created']}
              description={t['stepForm.desc.created']}
            />
          </Steps>
          <Form form={form} className={styles.form}>
            {current === 1 && (
              <Form.Item noStyle>
                <Form.Item
                  label={t['stepForm.basicInfo.name']}
                  required
                  field="basic.name"
                  rules={[
                    {
                      required: true,
                      message: t['stepForm.basicInfo.name.required'],
                    },
                    {
                      validator: (value: string, callback) => {
                        if (!/^[\u4e00-\u9fa5a-zA-Z0-9]{1,20}$/g.test(value)) {
                          callback(t['stepForm.basicInfo.name.placeholder']);
                        }
                      },
                    },
                  ]}
                >
                  <Input
                    placeholder={t['stepForm.basicInfo.name.placeholder']}
                  />
                </Form.Item>
                <Form.Item
                  label={t['stepForm.basicInfo.channelType']}
                  required
                  initialValue="app"
                  field="basic.channelType"
                  rules={[
                    {
                      required: true,
                      message: t['stepForm.basicInfo.channelType.required'],
                    },
                  ]}
                >
                  <Select>
                    <Select.Option value="app">APP通用渠道</Select.Option>
                    <Select.Option value="site">网页通用渠道</Select.Option>
                    <Select.Option value="game">游戏通用渠道</Select.Option>
                  </Select>
                </Form.Item>
                <Form.Item
                  label={t['stepForm.basicInfo.time']}
                  required
                  field="basic.time"
                  rules={[
                    {
                      required: true,
                      message: t['stepForm.basicInfo.time.required'],
                    },
                  ]}
                >
                  <DatePicker.RangePicker style={{ width: '100%' }} />
                </Form.Item>
                <Form.Item
                  label={t['stepForm.basicInfo.link']}
                  required
                  extra={t['stepForm.basicInfo.link.tips']}
                  field="basic.link"
                  initialValue={'https://arco.design'}
                  rules={[{ required: true }]}
                >
                  <Input
                    placeholder={t['stepForm.basicInfo.link.placeholder']}
                  />
                </Form.Item>
              </Form.Item>
            )}
            {current === 2 && (
              <Form.Item noStyle>
                <Form.Item
                  label={t['stepForm.channel.source']}
                  required
                  field="channel.source"
                  rules={[
                    {
                      required: true,
                      message: t['stepForm.channel.source.required'],
                    },
                  ]}
                >
                  <Input
                    placeholder={t['stepForm.channel.source.placeholder']}
                  />
                </Form.Item>
                <Form.Item
                  label={t['stepForm.channel.media']}
                  required
                  field="channel.media"
                  rules={[
                    {
                      required: true,
                      message: t['stepForm.channel.media.required'],
                    },
                  ]}
                >
                  <Input
                    placeholder={t['stepForm.channel.media.placeholder']}
                  />
                </Form.Item>
                <Form.Item
                  label={t['stepForm.channel.keywords']}
                  required
                  field="channel.keywords"
                  initialValue={['今日头条', '火山']}
                  rules={[{ required: true }]}
                >
                  <InputTag />
                </Form.Item>
                <Form.Item
                  label={t['stepForm.channel.remind']}
                  required
                  initialValue={true}
                  field="channel.remind"
                  triggerPropName="checked"
                  rules={[{ required: true }]}
                >
                  <Switch />
                </Form.Item>

                <Form.Item
                  label={t['stepForm.channel.content']}
                  required
                  field="channel.content"
                  rules={[
                    {
                      required: true,
                      message: t['stepForm.channel.content.required'],
                    },
                  ]}
                >
                  <Input.TextArea
                    placeholder={t['stepForm.channel.content.placeholder']}
                  />
                </Form.Item>
              </Form.Item>
            )}
            {current !== 3 ? (
              <Form.Item label=" ">
                <Space>
                  {current === 2 && (
                    <Button
                      size="large"
                      onClick={() => setCurrent(current - 1)}
                    >
                      {t['stepForm.prev']}
                    </Button>
                  )}
                  {current !== 3 && (
                    <Button type="primary" size="large" onClick={toNext}>
                      {t['stepForm.next']}
                    </Button>
                  )}
                </Space>
              </Form.Item>
            ) : (
              <Form.Item noStyle>
                <Result
                  status="success"
                  title={t['stepForm.created.success.title']}
                  subTitle={t['stepForm.created.success.desc']}
                  extra={[
                    <Button
                      key="reset"
                      style={{ marginRight: 16 }}
                      onClick={viewForm}
                    >
                      {t['stepForm.created.success.view']}
                    </Button>,
                    <Button key="again" type="primary" onClick={reCreateForm}>
                      {t['stepForm.created.success.again']}
                    </Button>,
                  ]}
                />
              </Form.Item>
            )}
          </Form>
        </div>
        {current === 3 && (
          <div className={styles['form-extra']}>
            <Title heading={6}>{t['stepForm.created.extra.title']}</Title>
            <Paragraph type="secondary">
              {t['stepForm.created.extra.desc']}
              <Button type="text">{t['stepForm.created.extra.detail']}</Button>
            </Paragraph>
          </div>
        )}
      </Card>
    </div>
  );
}

export default StepForm;
