#!/usr/bin/env bash
# DebitMyData Analytics (analytics.debitmydata.io): installs everything, or updates it when run
# again.
#
#   sudo /opt/debitmydata-analytics/deploy/debitmydata/install.sh
#
# Before the first run (README.md, "Install"): a fresh Ubuntu 24.04 server; this repository at
# /opt/debitmydata-analytics; the DebitMyData Core binaries (debitmydatad, debitmydata-cli) in
# /usr/local/bin; the Cloudflare origin certificate in /etc/ssl/cloudflare/.
#
# What it sets up:
#   swap                   8 GB, when the server has less than 8 GB of memory: building bitviewd
#                          needs more than 4 GB
#   debitmydata.service    this server's own DebitMyData node (main chain, no wallet, outgoing
#                          connections only, RPC on 127.0.0.1); bitviewd reads its block files
#                          (group dmd, read only)
#   Rust                   the version the repository names (rust-toolchain.toml), in /opt/rust
#   bitviewd               built from this repository when it changed, in /usr/local/bin
#   dmd-analytics.service  bitviewd on 127.0.0.1:3110, its data in /var/lib/debitmydata-analytics
#   nginx                  the website, behind Cloudflare
#   ufw                    SSH, and ports 80 and 443 for Cloudflare only
#
# It keeps the answers, the node's chain and bitviewd's data.
set -euo pipefail
# A command that fails stops the install: say where, rather than stopping without a word.
trap 'printf "\nERROR: the install stopped at line %s: %s\nRun it again once that is fixed: it carries on with what is already done.\n" "$LINENO" "$BASH_COMMAND" >&2' ERR

APP=/opt/debitmydata-analytics
HERE=$APP/deploy/debitmydata
CONF=/etc/debitmydata-analytics
INSTALL_CONF=$CONF/install.conf
DATA=/var/lib/debitmydata-analytics
DMD_DATA=/var/lib/debitmydata
DMD_CONF=/etc/debitmydata/debitmydata.conf
RUST=/opt/rust

step() { printf '\n==> %s\n' "$*"; }
fail() { printf '\nERROR: %s\n' "$*" >&2; exit 1; }
setting() { grep -E "^$1=" "$INSTALL_CONF" 2>/dev/null | tail -1 | cut -d= -f2- || true; }
ask() {  # ask NAME "question" default: the answer kept in install.conf, asked only once
    local name="$1" question="$2" default="$3" value
    value="$(setting "$name")"
    if [ -z "$value" ]; then
        if [ -t 0 ]; then
            read -r -p "$question [$default]: " value
        fi
        value="${value:-$default}"
        echo "$name=$value" >>"$INSTALL_CONF"
    fi
    printf -v "$name" '%s' "$value"
}

# ------------------------------------------------------------------ checks
step "Checking"
[ "$(id -u)" -eq 0 ] || fail "run as root: sudo $0"
grep -q 'VERSION_ID="24.04"' /etc/os-release || fail "this script is for Ubuntu 24.04"
[ "$(cd "$(dirname "$0")" && pwd)" = "$HERE" ] ||
    fail "the repository must be at $APP (git clone ... $APP)"
for bin in debitmydatad debitmydata-cli; do
    [ -x /usr/local/bin/$bin ] ||
        fail "/usr/local/bin/$bin is missing: copy the DebitMyData Core binaries there first (README.md, step 2)"
done
install -d -m 750 "$CONF"
touch "$INSTALL_CONF"
chmod 600 "$INSTALL_CONF"
ask ANALYTICS_HOST "The website's address" "analytics.debitmydata.io"
ask CERT_NAME "The Cloudflare origin certificate's name in /etc/ssl/cloudflare/" "debitmydata.io"
CERT=/etc/ssl/cloudflare/$CERT_NAME.pem
CERT_KEY=/etc/ssl/cloudflare/$CERT_NAME.key
[ -s "$CERT" ] && [ -s "$CERT_KEY" ] ||
    fail "the Cloudflare origin certificate is missing: save it as $CERT and its private key as $CERT_KEY (README.md, step 3)"
openssl x509 -in "$CERT" -noout -checkhost "$ANALYTICS_HOST" | grep -q "does match" ||
    fail "$CERT does not cover $ANALYTICS_HOST: make one for *.$(echo "$ANALYTICS_HOST" | cut -d. -f2-) in Cloudflare"
chmod 644 "$CERT"
chmod 600 "$CERT_KEY"

# ------------------------------------------------------------------ packages
step "Installing packages"
export DEBIAN_FRONTEND=noninteractive NEEDRESTART_SUSPEND=1
apt-get update -q
apt-get install -y -q --no-install-recommends nginx ufw acl curl git ca-certificates \
    build-essential pkg-config \
    libevent-core-2.1-7t64 libevent-extra-2.1-7t64 libevent-pthreads-2.1-7t64 libsqlite3-0 libzmq5 >/dev/null
missing="$(ldd /usr/local/bin/debitmydatad | grep 'not found' || true)"
[ -z "$missing" ] || fail "debitmydatad needs libraries that are not installed: $missing"

# ------------------------------------------------------------------ swap, for the build
step "Swap"
memory_kb="$(awk '/^MemTotal:/{print $2}' /proc/meminfo)"
if [ "$memory_kb" -lt 8000000 ] && ! swapon --show=NAME --noheadings | grep -q .; then
    [ -e /swapfile ] || { fallocate -l 8G /swapfile && chmod 600 /swapfile && mkswap /swapfile >/dev/null; }
    swapon /swapfile
    grep -q '^/swapfile ' /etc/fstab || echo '/swapfile none swap sw 0 0' >>/etc/fstab
    echo "8 GB of swap on (/swapfile)."
else
    echo "Enough memory, or swap already on."
fi

# ------------------------------------------------------------------ users and folders
step "Users and folders"
id dmd >/dev/null 2>&1 || useradd --system --home-dir "$DMD_DATA" --shell /usr/sbin/nologin dmd
id analytics >/dev/null 2>&1 || useradd --system --home-dir "$DATA" --shell /usr/sbin/nologin analytics
usermod -aG dmd analytics
install -d -m 750 -o dmd -g dmd "$DMD_DATA"
install -d -m 750 -o root -g dmd /etc/debitmydata
install -d -m 750 -o analytics -g analytics "$DATA" "$DATA/data"
# The node makes its files readable by itself alone; bitviewd reads its block files. A default ACL
# on its folder makes what the node creates readable by its group (dmd), never writable: the
# files already there too.
setfacl -R -m g::rX "$DMD_DATA"
setfacl -R -d -m u::rwX,g::rX,o::- "$DMD_DATA"

# ------------------------------------------------------------------ the DebitMyData node
step "The DebitMyData node (main chain)"
if [ ! -s "$DMD_CONF" ]; then
    cat >"$DMD_CONF" <<EOF
# The analytics server's own DebitMyData node: the main chain, no wallet, RPC on this server
# only, outgoing connections only. bitviewd signs in with the cookie (group-readable) and reads
# the block files.
server=1
disablewallet=1
listen=0
rpcbind=127.0.0.1
rpcallowip=127.0.0.1
rpcport=18557
rpccookieperms=group
dbcache=300
# The network's seed nodes, as in DebitMyData Core's own default settings.
seednode=dnsseed.debitmydata.network
seednode=dnsseed2.debitmydata.network
$(for n in $(seq 1 10); do printf 'addnode=seed%s.debitmydata.network\n' "$n"; done)
EOF
fi
chown root:dmd "$DMD_CONF"
chmod 640 "$DMD_CONF"
grep -qE '^(regtest|testnet|signet)=1' "$DMD_CONF" && fail "$DMD_CONF is not for the main chain"
grep -qE '^prune=[1-9]' "$DMD_CONF" && fail "$DMD_CONF prunes the chain: bitviewd needs every block file"
cat >/usr/local/bin/dmd-cli <<'EOF'
#!/bin/sh
# debitmydata-cli for this server's node, as its service user (cookie sign-in).
exec runuser -u dmd -- /usr/local/bin/debitmydata-cli -conf=/etc/debitmydata/debitmydata.conf -datadir=/var/lib/debitmydata "$@"
EOF
chmod 755 /usr/local/bin/dmd-cli
install -m 644 "$HERE/files/debitmydata.service" /etc/systemd/system/debitmydata.service
systemctl daemon-reload
systemctl enable --now debitmydata >/dev/null

# ------------------------------------------------------------------ Rust and bitviewd
step "Rust"
export RUSTUP_HOME=$RUST/rustup CARGO_HOME=$RUST/cargo PATH=$RUST/cargo/bin:$PATH
if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
    install -d -m 755 "$RUST"
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs |
        sh -s -- -y -q --profile minimal --default-toolchain none --no-modify-path
fi
# The version rust-toolchain.toml names, once
(cd "$APP" && rustup toolchain install >/dev/null)
(cd "$APP" && rustc --version)

COMMIT="$(git -C "$APP" rev-parse HEAD)"
if [ "$(cat "$CONF/built" 2>/dev/null)" != "$COMMIT" ] || [ ! -x /usr/local/bin/bitviewd ]; then
    step "Building bitviewd (the first time on 2 vCPU: an hour or more; later, only what changed)"
    started=$SECONDS
    # Thin LTO, not Bitview's fat one: a fraction of the memory and time, for a small chain.
    (cd "$APP" && CARGO_PROFILE_RELEASE_LTO=thin CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
        cargo build --release --locked -p bitviewd)
    install -m 755 "$APP/target/release/bitviewd" /usr/local/bin/bitviewd
    echo "$COMMIT" >"$CONF/built"
    elapsed=$((SECONDS - started))
    printf 'Built in %d h %02d min.\n' $((elapsed / 3600)) $((elapsed % 3600 / 60))
else
    echo "bitviewd is already built from $COMMIT."
fi

# ------------------------------------------------------------------ the analytics service
step "The analytics service"
install -m 644 "$HERE/files/dmd-analytics.service" /etc/systemd/system/dmd-analytics.service
systemctl daemon-reload
systemctl enable dmd-analytics >/dev/null
systemctl restart dmd-analytics

# ------------------------------------------------------------------ nginx
step "nginx"
sed -e "s#@HOST@#$ANALYTICS_HOST#g" -e "s#@CERT@#$CERT#g" -e "s#@KEY@#$CERT_KEY#g" \
    "$HERE/files/nginx-site.conf" >/etc/nginx/sites-available/dmd-analytics
if [ ! -e /proc/net/if_inet6 ]; then  # no IPv6 on this server: listen on IPv4 only
    sed -i '/listen \[::\]/d' /etc/nginx/sites-available/dmd-analytics
fi
ln -sf /etc/nginx/sites-available/dmd-analytics /etc/nginx/sites-enabled/dmd-analytics
rm -f /etc/nginx/sites-enabled/default
nginx -t -q
systemctl enable nginx >/dev/null
systemctl reload nginx || systemctl restart nginx

# ------------------------------------------------------------------ the firewall
step "The firewall"
# The SSH port to keep open: the one this session came in on (Ubuntu 24.04 can listen through
# ssh.socket, which sshd -T does not show, and sshd -T can fail), else sshd's setting, else 22.
SSH_PORT="${SSH_CONNECTION##* }"
if ! [[ "$SSH_PORT" =~ ^[0-9]+$ ]]; then
    SSH_PORT="$( (sshd -T 2>/dev/null || true) | awk '/^port /{print $2; exit}')"
fi
ufw allow "${SSH_PORT:-22}/tcp" comment 'SSH' >/dev/null
ufw default deny incoming >/dev/null
ufw default allow outgoing >/dev/null
ufw --force enable >/dev/null
"$HERE/cloudflare.sh"

# ------------------------------------------------------------------ check
step "Checking"
for _ in $(seq 1 30); do
    dmd-cli getblockcount >/dev/null 2>&1 && break
    sleep 2
done
height="$(dmd-cli getblockcount 2>/dev/null || echo "not answering yet")"
echo "The node: block $height (it syncs the chain first; dmd-cli getblockchaininfo)."
systemctl is-active --quiet dmd-analytics || fail "dmd-analytics is not running: journalctl -u dmd-analytics -n 50"
echo "bitviewd: running. It waits for the node to be synced, reads every block, then answers"
echo "on https://$ANALYTICS_HOST (follow it: journalctl -fu dmd-analytics)."
answer="$(curl -sk --max-time 10 --resolve "$ANALYTICS_HOST:443:127.0.0.1" "https://$ANALYTICS_HOST/api/server/sync" || true)"
echo "Now: ${answer:-not answering yet (still syncing)}"
echo
echo "Done."
