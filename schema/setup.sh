#!/bin/bash

DBNAME=${1:-arcs}
USERNAME=${2:-arcs}

RED='\033[0;31m'
BLU='\033[0;34m'
GRN='\033[0;32m'
NC='\033[0m'


function sql_file_user {
    echo "SET client_min_messages TO WARNING;" "$(cat $1)" | sql_file_user_loud /dev/stdin
}

function sql_file_user_loud {
    psql -d $DBNAME -U $USERNAME -q -f $1
}

function sql_cmd {
    sql_cmd_loud "SET client_min_messages TO WARNING; $1"
}

function sql_cmd_loud {
    psql -d $DBNAME -q -c "$1"
}

function echo_err {
  echo -e $RED$1$NC
}

function echo_info {
  echo -e $BLU$1$NC
}

function silence {
  "$@" > /dev/null 2>&1
}
function no_stdout {
  "$@" > /dev/null
}

echo_info "Creating database $DBNAME..."
silence createdb $DBNAME || echo_err "Failed to create database $DBNAME. It may already exist."
echo

echo_info 'Adding `citext` and `uuid-ossp` extensions to the database...'
no_stdout sql_cmd "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";"
no_stdout sql_cmd "CREATE EXTENSION IF NOT EXISTS citext;"
no_stdout sql_cmd "COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';"
echo

echo_info "Setting up the $USERNAME role..."
silence sql_cmd "CREATE ROLE $USERNAME WITH LOGIN;" || echo_err $RED"Failed to create role $USERNAME. It may already exist."$NC
no_stdout sql_cmd "GRANT ALL PRIVILEGES ON DATABASE $DBNAME TO $USERNAME;"
no_stdout sql_cmd "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $USERNAME;"
echo

echo_info "Setting up the schema..."
sql_file_user schema/0.sql
sql_file_user schema/1.sql
sql_file_user schema/2.sql
echo

echo_info "Setting up views..."
sql_file_user schema/views/helper.sql
sql_file_user schema/views/chall.sql
sql_file_user schema/views/team.sql
sql_file_user schema/views/user.sql
echo

echo_info "Setting up misc functions..."
sql_file_user schema/functions.sql
echo

echo -e $GRN"Done!"$NC