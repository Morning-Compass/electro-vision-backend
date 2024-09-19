#!/bin/bash

schema_up_file="/app/schema-up.sql"
schema_down_file="/app/schema-down.sql"

if [[ ! -f "$schema_up_file" ]]; then
    echo "Error: Schema file '$schema_up_file' not found."
    exit 1
fi

if [[ ! -f "$schema_down_file" ]]; then
    echo "Error: Schema file '$schema_down_file' not found."
    exit 1
fi

migration_folder=$(find /app/migrations/ -maxdepth 1 -type d -name '*[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]*_users' | sort -r | head -n 1)

echo "Migration folder found: $migration_folder"

if [[ -z "$migration_folder" ]]; then
    echo "Error: No migration folder found with a date format in its name."
    exit 1
fi

destination_up_file="$migration_folder/up.sql"
destination_down_file="$migration_folder/down.sql"

cp "$schema_up_file" "$destination_up_file"
cp "$schema_down_file" "$destination_down_file"

echo "Schema files have been copied to the migration folder:"
echo "  - '$destination_up_file'"
echo "  - '$destination_down_file'"
