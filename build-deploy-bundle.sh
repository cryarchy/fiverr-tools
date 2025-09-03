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
cp target/release/fiverr-message-checker checker/app && \
tar -cvzf checker.tar.gz checker && \
rm -rf checker