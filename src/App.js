import React, { useState } from 'react'
import { Layout, Row, Col, Switch, Typography, Badge, Space, Select, Button } from 'antd'
import { FileTextOutlined, CodeOutlined, CaretRightOutlined, ReloadOutlined } from '@ant-design/icons'
import Editor from './editor/editor'
import './App.css'
import 'antd/dist/antd.css';

const { Header, Content } = Layout
const { Text, Title } = Typography
const { Option } = Select

const statusMap = {
  '-2': 'default',
  '-1': 'processing',
  '0': 'success',
  '1': 'warning',
  '2': 'warning',
  '3': 'error',
  '4': 'error',
  '5': 'error',
}

const statusDescMap = {
  '-2': '暂无输出 - NULL',
  '-1': '运行中 - Running',
  '0': '运行成功 - Success',
  '1': '运行超时 - Time Limit Exceeded',
  '2': '时间超限 - Time Limit Exceeded',
  '3': '内存超限 - Memory Limit Exceeded',
  '4': '运行错误 - Runtime Error',
  '5': '系统错误(请联系管理员) - System Error',
}

const languageDescMap = {
  'C': 'C (gcc 5.4.0)',
  'C++': 'C++ (g++ 5.4.0)',
  'Java': 'Java (OpenJDK 1.8)',
  'Python3': 'Python (3.5.3)',
  'JavaScript': 'JavaScript (Node 8.16.1)'
}

const languageModeMap = {
  'C': 'c',
  'C++': 'cpp',
  'Java': 'java',
  'Python3': 'python',
  'JavaScript': 'javascript'
}

const languageCodeMap = {
  'C': '#include <stdio.h>\n\nint main() {\n\tprintf("hello, world\\n");\n\treturn 0;\n}\n',
  'C++': '#include <iostream>\nusing namespace std;\nint main() {\n\tcout << "hello, world" << endl;\n\treturn 0;\n}\n',
  'Java': 'public class Main {\n\tpublic static void main(String[] args) {\n\t\tSystem.out.println("hello, world");\n\t}\n}\n',
  'Python3': 'print("hello, world")\n',
  'JavaScript': 'console.log("hello, world")\n'
}

const sourceEditor = React.createRef()
const inputEditor = React.createRef()
const outputEditor = React.createRef()
const consoleEditor = React.createRef()
let time = 0

function App() {
  const wide = document.documentElement.clientWidth >= 768
  const [execStatus, setExecStatus] = useState(-2)
  const [needInput, setNeedInput] = useState(wide)
  const [language, setLanguage] = useState(localStorage.getItem('CLIOUDE_LANG') || 'C++')
  const [code, setCode] = useState(localStorage.getItem('CLIOUDE_CODE') || languageCodeMap[language])

  const handleRun = (e) => {
    const code = sourceEditor.current.getValue()
    localStorage.setItem('CLIOUDE_CODE', code)
    outputEditor.current && outputEditor.current.appendValue(`output${++time}\n`)
    consoleEditor.current && consoleEditor.current.appendValue(`output${++time}\n`)
    console.log(code)
  }

  return (
    <Layout>
      <Header style={{ height: 48 }}>
        <Space className="logo">
          Clioude
          {wide && <Button type="link" ghost onClick={() => sourceEditor.current.setValue(languageCodeMap[language])}>
            <ReloadOutlined />
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
              />
            </div>
          </Col>}
        </Row>
      </Content>
    </Layout>
  )
}

export default App;
