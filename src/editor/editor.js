import React from 'react'
import MonacoEditor from 'react-monaco-editor'

class Editor extends React.Component {

  editorDidMountWrapper(runFunc) {
    return (editor, monaco) => {
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
  
  render() {
    const { code, language, readOnly, handleRun, onChange } = this.props
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
        value={code || ''}
        options={options}
        onChange={onChange}
        editorDidMount={this.editorDidMountWrapper(handleRun)}
      />
    )
  }
}

export default Editor;