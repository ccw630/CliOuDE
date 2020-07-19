set -e

mkdir -p /tmp/ls && touch /tmp/ls/Main.java

echo "IndentWidth: 4" > /tmp/ls/.clang-format

exec python3 app.py