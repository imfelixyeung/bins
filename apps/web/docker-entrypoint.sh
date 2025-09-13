#!/usr/bin/env bash

# modified from
# https://github.com/docker-library/postgres/blob/master/17/alpine3.22/docker-entrypoint.sh

# usage: file_env VAR [DEFAULT]
#    ie: file_env 'XYZ_DB_PASSWORD' 'example'
# (will allow for "$XYZ_DB_PASSWORD_FILE" to fill in the value of
#  "$XYZ_DB_PASSWORD" from a file, especially for Docker's secrets feature)
file_env() {
  local var="$1"
  local fileVar="${var}_FILE"
  local def="${2:-}"
  if [ "${!var:-}" ] && [ "${!fileVar:-}" ]; then
    mysql_error "Both $var and $fileVar are set (but are exclusive)"
  fi
  local val="$def"
  if [ "${!var:-}" ]; then
    val="${!var}"
  elif [ "${!fileVar:-}" ]; then
    val="$(< "${!fileVar}")"
  fi
  export "$var"="$val"
  unset "$fileVar"
}

docker_setup_env() {
	file_env 'UPSTASH_REDIS_REST_URL'
	file_env 'UPSTASH_REDIS_REST_TOKEN'
	file_env 'RAPID_API_PROXY_SECRET'
	file_env 'BASE_URL'
}

_main() {
  docker_setup_env

  exec "$@"
}


_main "$@"
