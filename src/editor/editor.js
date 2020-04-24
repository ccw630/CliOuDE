import React from 'react'
import MonacoEditor from 'react-monaco-editor'

class Editor extends React.Component {

  constructor(props) {
    super(props);
    this.state = {
      value: props.code || '',
    }
  }

  editorDidMountWrapper(runFunc) {
    return (editor, monaco) => {
      this.editor = editor
      console.log('editorDidMount', editor)
      console.log(this.state)
      editor.addAction({
        id: 'run',
        label: 'run',
        keybindings: [monaco.KeyMod.Alt | monaco.KeyCode.KEY_R],
        run: runFunc
      })
      editor.focus()
    }
  }

  getValue = () => this.state.value
  setValue = (value) => this.setState({ value })
  appendValue = (value) => this.setState({ value: this.state.value + value })
  
  onChange = (newValue, e) => {
    this.setState({ value: newValue })
    // console.log('onChange', newValue, e)
  }

  render() {
    const { language, readOnly, handleRun } = this.props
    const { value } = this.state
    const options = {
      selectOnLineNumbers: true,
      minimap: { enabled: false },
      automaticLayout: true,
      fontSize: "14px",
      wordBasedSuggestions: language !== "plaintext",
      readOnly: readOnly || false,
      contextmenu: false,
    }
    return (
      <MonacoEditor
        language={language}
        value={value || ''}
        options={options}
        onChange={this.onChange}
        editorDidMount={this.editorDidMountWrapper(handleRun)}
      />
    )
  }
}

export default Editor;