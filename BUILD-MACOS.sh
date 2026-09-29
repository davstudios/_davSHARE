#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"
npm install --no-audit --no-fund
npm run bundle
