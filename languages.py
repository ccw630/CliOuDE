default_env = ["LANG=en_US.UTF-8", "LANGUAGE=en_US:en", "LC_ALL=en_US.UTF-8"]

_bash_lang_config = {
    "compile": {
        "src_name": "main.sh",
        "exe_name": "main_e.sh",
        "max_cpu_time": 5000,
        "max_real_time": 10000,
        "max_memory": -1,
        "compile_command": "/bin/cp {src_path} {exe_path}",
    },
    "run": {
        "command": "/usr/bin/bash {exe_path}",
        "seccomp_rule": None,
        "env": default_env
    }
}

_c_lang_config = {
    "compile": {
        "src_name": "main.c",
        "exe_name": "main",
        "max_cpu_time": 3000,
        "max_real_time": 10000,
        "max_memory": 512 * 1024 * 1024,
        "compile_command": "/usr/bin/gcc -DONLINE_JUDGE -O2 -w -fmax-errors=3 -std=c11 {src_path} -lm -o {exe_path}",
    },
    "run": {
        "command": "{exe_path}",
        "seccomp_rule": "c_cpp",
        "env": default_env
    }
}

_cpp_lang_config = {
    "compile": {
        "src_name": "main.cpp",
        "exe_name": "main",
        "max_cpu_time": 10000,
        "max_real_time": 20000,
        "max_memory": 1024 * 1024 * 1024,
        "compile_command": "/usr/bin/g++ -DONLINE_JUDGE -O2 -w -fmax-errors=3 -std=c++17 {src_path} -lm -o {exe_path}",
    },
    "run": {
        "command": "{exe_path}",
        "seccomp_rule": "c_cpp",
        "env": default_env
    }
}

_java_lang_config = {
    "compile": {
        "src_name": "Main.java",
        "exe_name": "Main",
        "max_cpu_time": 5000,
        "max_real_time": 10000,
        "max_memory": -1,
        "compile_command": "/usr/bin/javac {src_path} -d {exe_dir} -encoding UTF8"
    },
    "run": {
        "command": "/usr/bin/java -cp {exe_dir} -XX:MaxRAM={max_memory}k -Djava.security.manager -Dfile.encoding=UTF-8 "
                   "-Djava.security.policy==/etc/java_policy -Djava.awt.headless=true Main",
        "seccomp_rule": None,
        "env": default_env,
        "memory_limit_check_only": 1
    }
}


_py3_lang_config = {
    "compile": {
        "src_name": "main.py",
        "exe_name": "main_e.py",
        "max_cpu_time": 5000,
        "max_real_time": 10000,
        "max_memory": 256 * 1024 * 1024,
        "compile_command": "/bin/cp {src_path} {exe_path}",
    },
    "run": {
        "command": "/usr/bin/python3 {exe_path}",
        "seccomp_rule": "general",
        "env": default_env
    }
}

_js_lang_config = {
    "compile": {
        "src_name": "main.js",
        "exe_name": "main_e.js",
        "max_cpu_time": 5000,
        "max_real_time": 10000,
        "max_memory": -1,
        "compile_command": "/bin/cp {src_path} {exe_path}",
    },
    "run": {
        "command": "/usr/bin/node {exe_path}",
        "seccomp_rule": None,
        "env": default_env,
        "memory_limit_check_only": 1
    }
}

_bf_lang_config = {
    "compile": {
        "src_name": "main.bf",
        "exe_name": "main_e.bf",
        "max_cpu_time": 5000,
        "max_real_time": 10000,
        "max_memory": 128 * 1024 * 1024,
        "compile_command": "/bin/cp {src_path} {exe_path}",
    },
    "run": {
        "command": "/usr/bin/brainfuck/brainfuck.py {exe_path}",
        "seccomp_rule": None,
        "env": default_env
    }
}

_kotlin_lang_config = {
    "compile": {
        "src_name": "Main.kt",
        "exe_name": "Main",
        "max_cpu_time": 15000,
        "max_real_time": 30000,
        "max_memory": -1,
        "compile_command": "/usr/bin/kotlinc {src_path} -d {exe_dir}"
    },
    "run": {
        "command": "/usr/bin/java -cp /usr/lib/kotlin/*:{exe_dir} -XX:MaxRAM={max_memory}k -Djava.security.manager -Dfile.encoding=UTF-8 "
                   "-Djava.security.policy==/etc/java_policy -Djava.awt.headless=true MainKt",
        "seccomp_rule": None,
        "env": default_env,
        "memory_limit_check_only": 1
    }
}

_scala_lang_config = {
    "compile": {
        "src_name": "Main.scala",
        "exe_name": "Main",
        "max_cpu_time": 15000,
        "max_real_time": 30000,
        "max_memory": -1,
        "compile_command": "/usr/bin/scalac {src_path} -d {exe_dir} -encoding UTF8 -language:postfixOps"
    },
    "run": {
        "command": "/usr/bin/java -cp /usr/lib/scala/scala-library.jar:{exe_dir} -XX:MaxRAM={max_memory}k -Djava.security.manager -Dfile.encoding=UTF-8 "
                   "-Djava.security.policy==/etc/java_policy -Djava.awt.headless=true Main",
        "seccomp_rule": None,
        "env": default_env,
        "memory_limit_check_only": 1
    }
}

languages = {
    "Bash": _bash_lang_config,
    "C": _c_lang_config,
    "C++": _cpp_lang_config,
    "Java": _java_lang_config,
    "Python3": _py3_lang_config,
    "JavaScript": _js_lang_config,
    "Kotlin": _kotlin_lang_config,
    "Scala": _scala_lang_config,
    "Brainfuck": _bf_lang_config
}
