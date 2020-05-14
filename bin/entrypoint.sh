DB_URL_RAW="$(echo postgresql://clioude:clioude@localhost/clioude | sed -e 's/\//\\\//g')"
DB_URL_SUB="$(echo $DB_URL | sed -e 's/\//\\\//g')"
sed -i "s/$DB_URL_RAW/$DB_URL_SUB/g" ../alembic.ini

alembic upgrade head

exec python3 ../app.py