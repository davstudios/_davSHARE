#!/usr/bin/env bash
set -e
sudo find /etc/apt/sources.list.d -maxdepth 1 -type f \( -name '*microsoft*' -o -name '*azure-cli*' \) -print -delete 2>/dev/null || true
sudo apt-get update -o Acquire::Retries=3
sudo apt-get install -y --no-install-recommends libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

