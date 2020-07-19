FROM python:3.8-alpine3.10

ADD . /app
WORKDIR /app

RUN apk add --update --no-cache build-base postgresql-dev

RUN pip install --no-cache-dir tornado sqlalchemy psycopg2-binary alembic && \
    apk del build-base --purge

ENTRYPOINT /app/bin/entrypoint.sh