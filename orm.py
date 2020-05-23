from sqlalchemy import create_engine, Column, Text, Integer, Float, DateTime, func
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

from datetime import datetime, timedelta

import os

engine = create_engine(os.getenv('DB_URL'))
Base = declarative_base()
session = sessionmaker(bind=engine)
db = session()

class Worker(Base):

    __tablename__ = 'workers'

    id = Column(Integer, primary_key=True)
    hostname = Column(Text, nullable=False, index=True)
    version = Column(Text, nullable=False)
    cpu_core = Column(Integer, nullable=False)
    memory_usage = Column(Float, nullable=False)
    cpu_usage = Column(Float, nullable=False)
    last_heartbeat = Column(DateTime, nullable=False)
    create_time = Column(DateTime, nullable=False, default=datetime.now)
    task_number = Column(Integer, nullable=False, default=0)
    service_url = Column(Text, nullable=False)

    @classmethod
    def choose_worker(cls):
        try:
            db.begin(subtransactions=True)
            worker = db.query(cls).filter(cls.last_heartbeat + timedelta(seconds=6) >= datetime.now()).order_by(cls.task_number).first()
            if not worker:
                db.commit()
                return None
            worker.task_number += 1
            db.add(worker)
            db.commit()
        except Exception as e:
            db.rollback()
            raise e
        return worker
    

    @classmethod
    def upsert_worker(cls, hostname, version, cpu_core, memory_usage, cpu_usage, service_url):
        try:
            worker = db.query(cls).filter(cls.hostname == hostname).first()
            if not worker:
                worker = cls(hostname=hostname)
            worker.version = version
            worker.cpu_core = cpu_core
            worker.memory_usage = memory_usage
            worker.cpu_usage = cpu_usage
            worker.service_url = service_url
            worker.last_heartbeat = datetime.now()
            db.add(worker)
            db.commit()
        except Exception as e:
            db.rollback()
            raise e

    @classmethod
    def cull_worker(cls):
        try:
            db.query(cls).filter(cls.last_heartbeat + timedelta(seconds=600) < datetime.now()).delete()
            db.commit()
        except Exception as e:
            db.rollback()
            raise e