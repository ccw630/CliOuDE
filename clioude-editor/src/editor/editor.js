import React from 'react'
import MonacoEditor from 'react-monaco-editor'
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'
import { listen } from 'vscode-ws-jsonrpc';
import {
    MonacoLanguageClient, CloseAction, ErrorAction,
    MonacoServices, createConnection
} from 'monaco-languageclient';
import ReconnectingWebSocket from 'reconnecting-websocket'
import * as scala from './languages/scala'

const lsp_supported_languages = ['python', 'c', 'cpp', 'shell', 'java']
const lsp_inmemory_languages = ['python', 'shell']

monaco.languages.register({ id: 'scala' })
monaco.languages.setMonarchTokensProvider('scala', scala.language)
monaco.languages.setLanguageConfiguration('scala', scala.conf)

class Editor extends React.Component {

  constructor(props) {
    super(props);
    this.state = {
      lastPos: { line: 1, column: 1 },
      undo: false,
      input: '',
      appending: false,
    }
    this.languageSocket = null
    this.languageSocketListened = false
  }

  invalidPosition = (line, column) => (line === this.state.lastPos.line && column < this.state.lastPos.column) || line < this.state.lastPos.line

  createLanguageClient = () => {
    if (this.languageSocketListened) this.languageSocket.close()
    const _language = this.props.language
    if (!lsp_supported_languages.includes(_language)) {
      return
    }
    if (!lsp_inmemory_languages.includes(_language)) {
      MonacoServices.install(this.editor, {rootUri: '/tmp/ls/'})
      const uri = monaco.Uri.parse(`/tmp/ls/Main.${_language}`)
      this.editor.setModel(monaco.editor.getModel(uri) || monaco.editor.createModel(this.props.code, _language, uri))
    } else {
      MonacoServices.install(this.editor)
    }

    const url = `ws://${process.env.NODE_ENV === 'development' ? 'localhost:8998' : window.location.host}/lsp/${_language}`
    this.languageSocket = createWebSocket(url)
   
    // listen when the web socket is opened
    listen({
      webSocket: this.languageSocket,
      onConnection: connection => {
          // create and start the language client
          const languageClient = createLanguageClient(connection)
          const disposable = languageClient.start()
          connection.onClose(() => {
            disposable.dispose()
            this.languageSocketListened = false
          })
          this.languageSocketListened = true
      }
    })

    function createLanguageClient(connection) {
      return new MonacoLanguageClient({
        name: "A Language Client",
        clientOptions: {
          // use a language id as a document selector
          documentSelector: [_language],
          // disable the default error handler
          errorHandler: {
            error: () => ErrorAction.Continue,
            closed: () => CloseAction.DoNotRestart
          }
        },
        // create a language client connection from the JSON RPC connection on demand
        connectionProvider: {
          get: (errorHandler, closeHandler) => {
              return Promise.resolve(createConnection(connection, errorHandler, closeHandler))
          }
        }
      })
    }

    function createWebSocket(url) {
      const socketOptions = {
          maxReconnectionDelay: 10000,
          minReconnectionDelay: 1000,
          reconnectionDelayGrowFactor: 1.3,
          connectionTimeout: 10000,
          maxRetries: Infinity,
          debug: false
      }
      return new ReconnectingWebSocket(url, [], socketOptions)
    }
  }

  editorDidMount = (editor) => {
    this.editor = editor
    
    // editor.onDidChangeCursorPosition(e => {
    //   if (this.invalidPosition(e.position.lineNumber, e.position.column)) {
    //     this.editor.setPosition({ lineNumber: this.state.lastPos.line, column: this.state.lastPos.column })
    //   }
    // })

    this.createLanguageClient()

    editor.setValue(this.props.code || '')
    editor.focus()
  }

  reformat = () => this.editor.trigger('', 'editor.action.formatDocument')
  getValue = () => this.editor.getValue()
  setValue = (value) => this.editor.setValue(value)
  clear = () => {
    this.setState({ lastPos: { line: 1, column: 1 }})
    this.setValue('')
  }
  appendValue = (text) => {
    const range = new monaco.Range(
        this.state.lastPos.line,
        this.state.lastPos.column,
        this.state.lastPos.line,
        this.state.lastPos.column,
    )

    this.setState({ appending: true }, () => {
      this.editor.executeEdits('', [
        { range, text, forceMoveMarkers: true }
      ], (op) => this.setState({ lastPos: { line: op[0].range.endLineNumber, column: op[0].range.endColumn }}))
      this.editor.pushUndoStop()
    })
  }
  
  onChange = (newValue, e) => {
    if (!this.props.consoleMode) return
    const invalid = e.changes.some(x => !this.state.appending && x.range && (this.invalidPosition(x.range.startLineNumber, x.range.startColumn) || this.props.readOnly))
    if (e.isUndoing && !this.state.appending && invalid) {
      this.setState({ appending: true }, () => this.editor.trigger('', 'redo'))
    } else if (invalid) {
      this.setState({ appending: true }, () => this.editor.trigger('', 'undo'))
    } else {
      if (!this.state.appending && e.changes.some(x => x.text === '\n' || x.text === '\r')) {
        const endColumn = 1, endLineNumber = this.editor.getModel().getLineCount()
        const startColumn = this.state.lastPos.column, startLineNumber = this.state.lastPos.line
        this.props.sendInput(this.editor.getModel().getValueInRange({ endColumn, endLineNumber, startColumn, startLineNumber }))
        this.setState({ lastPos: { line: endLineNumber, column: endColumn }})
      }
      this.setState({ appending: false })
    }
  }

  render() {
    const { language, readOnly } = this.props
    const options = {
      selectOnLineNumbers: true,
      minimap: { enabled: false },
      automaticLayout: true,
      fontSize: "14px",
      wordBasedSuggestions: !lsp_supported_languages.includes(language),
      contextmenu: false,
      readOnly: readOnly || false,
    }
    return (
      <MonacoEditor
        key={language}
        language={language}
        options={options}
        onChange={this.onChange}
        editorDidMount={this.editorDidMount}
      />
    )
  }
}

export default Editor;