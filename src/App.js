import React, { useState } from 'react'
import { Layout, Row, Col, Switch, Typography, Badge, Space, Select, Button } from 'antd'
import { FileTextOutlined, CodeOutlined, CaretRightOutlined, ReloadOutlined, AlignLeftOutlined } from '@ant-design/icons'
import Editor from './editor/editor'
import { statusMap, statusDescMap, languageDescMap, languageModeMap, languageCodeMap } from './scripts/constants'
import './App.css'
import 'antd/dist/antd.css';

const { Header, Content } = Layout
const { Text } = Typography
const { Option } = Select

const sourceEditor = React.createRef()
const inputEditor = React.createRef()
const outputEditor = React.createRef()
const consoleEditor = React.createRef()

function App() {
  const wide = document.documentElement.clientWidth >= 768
  const [execStatus, setExecStatus] = useState(-2)
  const [needInput, setNeedInput] = useState(wide)
  const [language, setLanguage] = useState(localStorage.getItem('CLIOUDE_LANG') || 'C++')
  const [code, setCode] = useState(localStorage.getItem('CLIOUDE_CODE') || languageCodeMap[language])

  const handleRun = (e) => {
    const code = sourceEditor.current.getValue()
    localStorage.setItem('CLIOUDE_CODE', code)
    outputEditor.current && outputEditor.current.clear()
    consoleEditor.current && consoleEditor.current.clear()
    setExecStatus(-1)
    // consoleEditor.current && consoleEditor.current.appendValue('output\n')
  }

  const sendInput = (input) => {
    console.log(input)
  }

  return (
    <Layout>
      <Header style={{ height: 48 }}>
        <Space className="logo">
          Clioude
          {wide && <Button type="link" ghost onClick={() => sourceEditor.current.setValue(languageCodeMap[language])}>
            <ReloadOutlined />
          </Button>}
          {false && <Button type="link" ghost onClick={() => sourceEditor.current.reformat()}>
            <AlignLeftOutlined />
          </Button>}
        </Space>
        <div className="right">
          <Space>
            <Select
              value={language}
              style={{ width: wide ? 330 : 120 }}
              onChange={value => {
                setLanguage(value)
                localStorage.setItem('CLIOUDE_LANG', value)
              }}
            >
              {Object.keys(languageDescMap).map(k => <Option value={k}>{languageDescMap[k]}</Option>)}
            </Select>
            <Button type="primary" icon={<CaretRightOutlined />} onClick={handleRun}>
              {wide && "运行(Alt + R)"}
            </Button>
          </Space>
        </div>
      </Header>
      <Content>
        <div id="sourceEditor">
          <Editor
            language={languageModeMap[language]}
            code={code}
            handleRun={handleRun}
            isSourceEditor={true}
            ref={sourceEditor}
          />
        </div>
        <Row>
          {needInput && wide && <Col span={12}>
            <div id="inputLabel">
              <Space>
                <FileTextOutlined />
                <Text>输入 Input</Text>
              </Space>
            </div>
          </Col>}
          <Col span={needInput ? 12 : 24}>
            <Row id="outputLabel">
              <Col span={12}>
                <Space>
                  <CodeOutlined />
                  <Text>{needInput ? "输出 Output" : "控制台 Console"}</Text>
                </Space>
              </Col>
              <Col span={12} style={{"textAlign": "right"}}>
                <Space>
                  {wide && <Switch
                    checkedChildren="输入: 开"
                    unCheckedChildren="输入: 关"
                    checked={needInput}
                    disabled={execStatus === -1}
                    onChange={(checked, e) => setNeedInput(checked)}
                  />}
                  <Badge status={statusMap[execStatus]} text={statusDescMap[execStatus]} />
                </Space>
              </Col>
            </Row>
          </Col>
        </Row>
        <Row>
          {needInput && wide && <>
            <Col span={12}>
              <div id="inputEditor">
                <Editor
                  language="plaintext"
                  handleRun={handleRun}
                  sendInput={sendInput}
                  ref={inputEditor}
                />
              </div>
            </Col>
            <Col span={12}>
              <div id="outputEditor">
                <Editor
                  language="plaintext"
                  readOnly={true}
                  handleRun={handleRun}
                  ref={outputEditor}
                />
              </div>
            </Col>
          </>}
          {!needInput && <Col span={24}>
            <div id="outputEditor">
              <Editor
                ref={consoleEditor}
                language="plaintext"
                handleRun={handleRun}
                sendInput={sendInput}
              />
            </div>
          </Col>}
        </Row>
      </Content>
    </Layout>
  )
}

export default App;
