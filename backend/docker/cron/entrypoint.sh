#!/bin/sh
# Cron hands jobs a minimal environment of its own, so anything the sync needs
# (DATABASE_URL, RUST_LOG) has to be written into the crontab. Generating it here
# keeps DATABASE_URL configured in exactly one place, in compose.
set -eu

: "${DATABASE_URL:?DATABASE_URL must be set}"
: "${SYNC_COMMAND:?SYNC_COMMAND must be set}"
SCHEDULE="${SCHEDULE:-0 4 * * *}"
# Files in /etc/cron.d use the same layout as /etc/crontab, so the user the job
# runs as is a required field. Both the user and the binary are spelled out
# rather than left to cron to resolve, because omitting the user field makes
# cron silently run the next word as the command.
SYNC_USER="${SYNC_USER:-backend}"
SYNC_BIN="${SYNC_BIN:-/usr/local/bin/backend}"
LOG_FILE="${LOG_FILE:-/var/log/sync.log}"

# The values are interpolated into a crontab, so a newline would let them inject
# extra entries.
case "$DATABASE_URL$SYNC_COMMAND$SCHEDULE${RUST_LOG:-}" in
    *'
'*)
        echo "cron settings must not contain newlines" >&2
        exit 1
        ;;
esac

cat >/etc/cron.d/sync <<EOF
SHELL=/bin/sh
PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
MAILTO=""
DATABASE_URL=${DATABASE_URL}
RUST_LOG=${RUST_LOG:-backend=info}

# Fires in the container's local time, set by TZ in compose.
${SCHEDULE} ${SYNC_USER} ${SYNC_BIN} ${SYNC_COMMAND} >>${LOG_FILE} 2>&1
EOF

chmod 0644 /etc/cron.d/sync

# Jobs run unprivileged, but /proc/1/fd is root-only, so a job cannot write to
# PID 1's stdout directly. Output therefore goes to a world-writable log file
# which is tailed into PID 1, keeping it visible in `docker compose logs`.
touch "$LOG_FILE"
chmod 0666 "$LOG_FILE"

echo "scheduled '${SCHEDULE} ${SYNC_USER} ${SYNC_BIN} ${SYNC_COMMAND}' ($(date))" >&2

# cron runs in the background so that tail owns PID 1 and relays the log.
cron -f &
exec tail -n 0 -F "$LOG_FILE"