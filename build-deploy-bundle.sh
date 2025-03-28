#!/bin/bash

cargo build --release && \
mkdir -p checker && \
cp THIRD_PARTY_NOTICES.chromedriver checker/ && \
cp LICENSE.chromedriver checker/ && \
cp chromedriver_PATCHED checker/ && \
cp chromedriver checker/ && \
cp app-config.yaml checker/ && \
cp .env checker/ && \
cp -r migrations checker/ && \
mkdir -p checker/postgres_db && \
cp postgres_db/docker-compose.yml checker/postgres_db && \
cp postgres_db/env-vars checker/postgres_db && \
cp target/release/fiverr-message-checker checker/app && \
tar -cvzf checker.tar.gz checker && \
rm -rf checker