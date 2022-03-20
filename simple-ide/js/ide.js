let sourceEditor, inputEditor, outputEditor
let selectLanguageBtn, runBtn, vimCheckBox
let statusLine
let outputListener

function reset() {
  runBtn.removeAttribute("disabled");
  outputListener.dispose()
}

function toggleVim() {
  const keyMap = vimCheckBox.checked ? "vim" : "default"
  localStorageSetItem("keyMap", keyMap);
  sourceEditor.setOption("keyMap", keyMap);
  focusAndSetCursorAtTheEnd();
}

function saveCode() {
  localStorageSetItem("codeContent", sourceEditor.getValue())
}

function createSession() {
  if (sourceEditor.getValue().trim() == "") {
    alert("代码不能为空!");
    return;
  } else {
    runBtn.setAttribute("disabled", "disabled");
  }

  let sourceValue = sourceEditor.getValue()
  let inputValue = inputEditor.getValue()
  let language = selectLanguageBtn.selectedOptions[0].value
  const sessionReq = {
    code: sourceValue,
    language: language
  }

  const headers = new Headers();
  headers.append("Content-Type", "application/json");
  const requestOptions = {
    method: 'POST',
    headers: headers,
    body: JSON.stringify(sessionReq),
    redirect: 'follow',
  }

  const timeStart = performance.now();

  fetch("http://localhost:8080/session", requestOptions)
  .then(response => {
    if (response.status !== 200) {
      return response.text()
    }
    return response.json()
  })
  .then(result => {
    if (!result.session_id) {
      alert(result)
      reset()
      return
    }
    const sessionId = result.session_id
    const io = new WebSocket(`ws://localhost:8080/endpoint-io?session_id=${sessionId}`)
    outputEditor.write('\x1b[H\x1b[2J') // clear terminal
    io.onopen =  () => {
      if (!!inputValue) {
        io.send(inputValue)
      }
      outputListener = outputEditor.onData(data => {
        data = data.replaceAll("\r", "\n")
        outputEditor.write(data)
        io.send(data)
      })
    }
    io.onmessage = (e) => {
      appendOutput(e.data)
    }
    io.onclose = () => {
      statusLine.innerHTML = 'OK'
      reset()
      console.log("It took " + (performance.now() - timeStart) + " ms to get submission result.");
    }

    const st = new WebSocket(`ws://localhost:8080/endpoint-st?session_id=${sessionId}`)
    st.onmessage = (e) => {
      data = JSON.parse(e.data)
      if (data.type === 'status') {
        statusLine.innerHTML = data.desc
      } else if (data.type === 'exit') {
        appendOutput(`\n[INFO] Exited with code ${data.desc}.`)
      }

    }

  })
  .catch(error => {
    reset()
    alert('Connection refused')
    console.error(error)
  })
}

function appendOutput(output) {
  outputEditor.write(output)
}

function setEditorMode() {
  sourceEditor.setOption("mode", selectLanguageBtn.selectedOptions[0].getAttribute('mode'))
}

function focusAndSetCursorAtTheEnd() {
  sourceEditor.focus();
  sourceEditor.setCursor(sourceEditor.lineCount(), 0);
}

function insertTemplate() {
  const value = selectLanguageBtn.selectedOptions[0].value
  sourceEditor.setValue(sources[value])
  focusAndSetCursorAtTheEnd();
  sourceEditor.markClean();
}

function loadDefaultLanguage() {
  selectLanguageBtn.selectedIndex = 0 // C++
  setEditorMode();
  if (sourceEditor.getValue() === "") {
    insertTemplate();
  }
}

function initializeElements() {
  selectLanguageBtn = document.getElementById("selectLanguageBtn")
  runBtn = document.getElementById("runBtn")
  vimCheckBox = document.getElementById("vimCheckBox")
  statusLine = document.getElementById("statusLine")
}

function localStorageSetItem(key, value) {
  try {
    localStorage.setItem(key, value);
  } catch (ignorable) {
  }
}

function localStorageGetItem(key) {
  try {
    return localStorage.getItem(key);
  } catch (ignorable) {
    return null;
  }
}

window.onload = () => {
  initializeElements();

  sourceEditor = CodeMirror(document.getElementById("sourceEditor"), {
    lineNumbers: true,
    indentUnit: 4,
    indentWithTabs: true,
    showCursorWhenSelecting: true,
    matchBrackets: true,
    autoCloseBrackets: true,
    value: localStorageGetItem("codeContent") || '',
    keyMap: localStorageGetItem("keyMap") || "default",
    extraKeys: {
      "Tab": function(cm) {
        const spaces = Array(cm.getOption("indentUnit") + 1).join(" ");
        cm.replaceSelection(spaces);
      }
    }
  })

  inputEditor = CodeMirror(document.getElementById("inputEditor"), {
    lineNumbers: true,
    mode: "plain"
  })

  outputEditor = new Terminal({
    convertEol: true,
    theme: { background: "#f8f8f8", foreground: "#000000", selection: "#001528", cursor: "#000000" },
    fontSize: 16,
    lineHeight: 1.375,
  })
  outputEditor.open(document.getElementById('outputEditor'));

  if (localStorageGetItem("keyMap") == "vim") {
    vimCheckBox.checked = true
    toggleVim()
  }

  loadDefaultLanguage();

  selectLanguageBtn.onchange = () => {
    if (sourceEditor.isClean()) {
      insertTemplate();
    }
    setEditorMode();
  }

  window.onkeydown = (e) => {
    const keyCode = e.keyCode || e.which;
    if (e.altKey && keyCode === 82) { // ⌥ + R
      e.preventDefault();
      createSession();
    }
  }

  runBtn.onclick = createSession

  CodeMirror.commands.save = saveCode

  vimCheckBox.onchange = toggleVim
}

// Template Sources
const cSource = "\
#include <stdio.h>\n\
\n\
int main() {\n\
\tprintf(\"hello, world\\n\");\n\
\treturn 0;\n\
}\n";

const cppSource = "\
#include <iostream>\n\
using namespace std;\n\
int main() {\n\
\tcout << \"hello, world\" << endl;\n\
\treturn 0;\n\
}\n";

const javaSource = "\
public class Main {\n\
\tpublic static void main(String[] args) {\n\
\t\tSystem.out.println(\"hello, world\");\n\
\t}\n\
}\n";

const python3Source = "print(\"hello, world\")\n";

const python2Source = "print \"hello, world\"\n";

const javascriptSource = "console.log(\"hello, world\")\n"

const kotlinSource = "\
fun main() {\n\
\tprintln(\"hello, world\")\n\
}\n"

const scalaSource = "\
object Main extends App {\n\
\tprintln(\"hello, world\")\n\
}\n"

const sources = {
  "C": cSource,
  "C++": cppSource,
  "Java": javaSource,
  "Python3": python3Source,
  "Python2": python2Source,
  "JavaScript": javascriptSource,
  "Kotlin": kotlinSource,
  "Scala": scalaSource
};

window.onbeforeunload = (e) => {
  return '你代码保存了吗'
}

function autoFormat() {
    for (let i=0; i<sourceEditor.lineCount(); i++) {
    	sourceEditor.indentLine(i);
    }
}
