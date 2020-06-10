export const statusMap = {
  '-4': 'default',
  '-3': 'error',
  '-2': 'default',
  '-1': 'processing',
  '0': 'success',
  '1': 'warning',
  '2': 'warning',
  '3': 'error',
  '4': 'error',
  '5': 'error',
}

export const statusDescMap = {
  '-4': '已停止 - Terminated',
  '-3': '编译错误 - Compile Error',
  '-2': '暂无输出 - NULL',
  '-1': '运行中 - Running',
  '0': '运行成功 - Success',
  '1': '运行超时 - Time Limit Exceeded',
  '2': '时间超限 - Time Limit Exceeded',
  '3': '内存超限 - Memory Limit Exceeded',
  '4': '运行错误 - Runtime Error',
  '5': '系统错误(请联系管理员) - System Error',
}

export const languageDescMap = {
  'Bash': 'Bash (GNU bash 5.0.16)',
  'C': 'C (gcc 9.3.0)',
  'C++': 'C++ (g++ 9.3.0)',
  'Java': 'Java (OpenJDK 11.0.7)',
  'Python3': 'Python (Python 3.8.2)',
  'JavaScript': 'JavaScript (Node.js v12.16.3)',
  'Kotlin': 'Kotlin (Kotlin 1.3.50)'
}

export const languageModeMap = {
  'Bash': 'shell',
  'C': 'c',
  'C++': 'cpp',
  'Java': 'java',
  'Python3': 'python',
  'JavaScript': 'javascript',
  'Kotlin': 'kotlin'
}

export const languageCodeMap = {
  'Bash': 'echo "hello, world"\n',
  'C': '#include <stdio.h>\n\nint main() {\n\tprintf("hello, world\\n");\n\treturn 0;\n}\n',
  'C++': '#include <iostream>\nusing namespace std;\nint main() {\n\tcout << "hello, world" << endl;\n\treturn 0;\n}\n',
  'Java': 'public class Main {\n\tpublic static void main(String[] args) {\n\t\tSystem.out.println("hello, world");\n\t}\n}\n',
  'Python3': 'print("hello, world")\n',
  'JavaScript': 'console.log("hello, world")\n',
  'Kotlin': 'fun main() {\n\tprintln("hello, world")\n}\n'
}
