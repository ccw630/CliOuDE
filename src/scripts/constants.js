export const statusMap = {
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
  'C': 'C (gcc 5.4.0)',
  'C++': 'C++ (g++ 5.4.0)',
  'Java': 'Java (OpenJDK 1.8)',
  'Python3': 'Python (3.5.3)',
  'JavaScript': 'JavaScript (Node 8.16.1)'
}

export const languageModeMap = {
  'C': 'c',
  'C++': 'cpp',
  'Java': 'java',
  'Python3': 'python',
  'JavaScript': 'javascript'
}

export const languageCodeMap = {
  'C': '#include <stdio.h>\n\nint main() {\n\tprintf("hello, world\\n");\n\treturn 0;\n}\n',
  'C++': '#include <iostream>\nusing namespace std;\nint main() {\n\tcout << "hello, world" << endl;\n\treturn 0;\n}\n',
  'Java': 'public class Main {\n\tpublic static void main(String[] args) {\n\t\tSystem.out.println("hello, world");\n\t}\n}\n',
  'Python3': 'print("hello, world")\n',
  'JavaScript': 'console.log("hello, world")\n'
}
