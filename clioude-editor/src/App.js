import React, { useState, useEffect } from 'react'
import { Layout, Row, Col, Switch, Typography, Badge, Space, Select, Button } from 'antd'
import { CloudServerOutlined, FileTextOutlined, CodeOutlined, CaretRightOutlined, CloseOutlined, ReloadOutlined } from '@ant-design/icons'

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

const runningStatus = [-1, -5, -6]

function App() {
  const wide = document.documentElement.clientWidth >= 768
  const [execStatus, setExecStatus] = useState(-2)
  const [needInput, setNeedInput] = useState(wide)
  const [language, setLanguage] = useState(localStorage.getItem('CLIOUDE_LANG') || 'C++')
  const [code, setCode] = useState(localStorage.getItem('CLIOUDE_CODE') || languageCodeMap[language])
  const [ws, setWs] = useState(null)
  const [running, setRunning] = useState(false)
  const [extraInfo, setExtraInfo] = useState('')

  const handleRun = (e) => {
    if (running) {
      ws && ws.close()
      setExecStatus(-4)
      setRunning(false)
    } else {
      const code = sourceEditor.current.getValue()
      localStorage.setItem('CLIOUDE_CODE', code)
      outputEditor.current && outputEditor.current.clear()
      consoleEditor.current && consoleEditor.current.clear()
      setWs(new WebSocket(`ws://${process.env.NODE_ENV === 'development' ? 'localhost:8081' : window.location.host}/api/run`))
      setExecStatus(-7)
      setRunning(true)
    }
    setExtraInfo('')
  }

  document.onkeydown = (e) => {
    var keyCode = e.keyCode || e.which;
    if (e.altKey && keyCode === (running ? 84 : 82)) {
      e.preventDefault();
      handleRun();
    }
  }

  useEffect(() => {
    if(ws) {
      ws.onmessage = msg => {
        const data = JSON.parse(msg.data)
        if (data.type === 'result') {
          const result = data.data.result
          if (!runningStatus.includes(result)) {
            setRunning(false)
            if (result === -3 || result === 5) {
              data.data.err && writeOutput(data.data.err.replace(/\/worker\/run\/\S+\//g, ''))
            } else {
              const time = (data.data.cpu_time === null ? "-" : data.data.cpu_time + "ms")
              const rtime = (data.data.real_time === null ? "-" : Math.floor(data.data.real_time / 1000) + "s")
              const memory = (data.data.memory === null ? "-" : Math.floor(data.data.memory / 1024 / 1024) + "MB")
              setExtraInfo(`, ${rtime} (CPU ${time}), ${memory}`)
            }
            let extra = ''
            if (data.data.exit_code) {
              extra += `\n[WARN] Exited with code ${data.data.exit_code}.`
            }
            if (data.data.signal) {
              extra += `\n[WARN] Killed by signal ${data.data.signal}.`
            }
            if (extra) setTimeout(() => writeOutput(extra), 200)
          }
          setExecStatus(result)
        } else if (data.type === 'output') {
          writeOutput(data.data)
        }
      }
      ws.onopen = () => {
        ws.send(JSON.stringify({
          type: 'code',
          data: {
            src: sourceEditor.current.getValue(),
            language,
            input_content: inputEditor.current ? inputEditor.current.getValue() : null
          }
        }))
      }
    }
  })

  const writeOutput = (output) => {
    outputEditor.current && outputEditor.current.appendValue(output)
    consoleEditor.current && consoleEditor.current.appendValue(output)
  }

  const sendInput = (input) => {
    if (input && !input.endsWith('\n')) input += '\n'
    ws.send(JSON.stringify({
      type: 'input',
      data: input
    }))
  }

  return (
    <Layout>
      <Header style={{ height: 48 }}>
        <Space className="logo">
          {wide && <CloudServerOutlined />}
          CliOuDE
          {wide && <Button id="reset" type="link" ghost onClick={() => sourceEditor.current.setValue(languageCodeMap[language])}>
            <ReloadOutlined />
          </Button>}
        </Space>
        <div className="rightHeader">
          <Space>
            {wide && <Switch
              checkedChildren="输入: 开"
              unCheckedChildren="输入: 关"
              checked={needInput}
              disabled={runningStatus.includes(execStatus)}
              size="big"
              onChange={(checked, e) => setNeedInput(checked)}
            />}
            <Select
              value={language}
              style={{ width: wide ? 330 : 100 }}
              id="languages"
              onChange={value => {
                setCode(sourceEditor.current.getValue())
                localStorage.setItem('CLIOUDE_CODE', sourceEditor.current.getValue())
                setLanguage(value)
                localStorage.setItem('CLIOUDE_LANG', value)
              }}
            >
              {Object.keys(languageDescMap).map(k => <Option value={k} id={`language-${k}`}>{languageDescMap[k]}</Option>)}
            </Select>
            <Button type="primary" id="trigger" icon={running ? <CloseOutlined /> : <CaretRightOutlined />} onClick={handleRun} danger={running}>
              {wide && (running ? "停止(⌥ + T)" : "运行(⌥ + R)")}
            </Button>
          </Space>
        </div>
      </Header>
      <Content>
        <div id="sourceEditor">
          <Editor
            language={languageModeMap[language]}
            code={code}
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
                  <Badge id="status" status={statusMap[execStatus]} text={statusDescMap[execStatus] + extraInfo} />
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
                  sendInput={()=>{}}
                  ref={inputEditor}
                />
              </div>
            </Col>
            <Col span={12}>
              <div id="outputEditor">
                <Editor
                  language="plaintext"
                  consoleMode={true}
                  sendInput={()=>{}}
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
                consoleMode={true}
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
