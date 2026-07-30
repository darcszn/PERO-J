# Solve #106 — Database Backup Strategy

## Summary

This PR adds a complete database backup and restore strategy for the PERO-J Soroban Smart Block Explorer, addressing issue #106. Previously, the project had no documentation or automation for PostgreSQL backups, putting all decoded event history and registered ABI metadata at risk of permanent loss on database failure.

## Changes

### 1. New: `scripts/backup.sh`

A production-ready backup script using `pg_dump` that:

- Produces a plain-text SQL dump of the `soroban_explorer` database
- Uses configurable environment variables (`PGHOST`, `PGPORT`, `PGUSER`, `PGDATABASE`, `PGPASSWORD`) for connection flexibility
- Compresses output-free plain format for reliability
- Cleans up backups older than 30 days automatically
- Logs all operations to `logs/backup.log` for auditing and debugging
- Uses `set -euo pipefail` for strict error handling
- Is executable (`chmod +x`)

### 2. Updated: `README.md` — Database Backup Section

Adds a comprehensive **Database Backup** section covering:

- **Local Backup Script** — instructions for using `scripts/backup.sh` manually
- **Environment Variables** — full table of configurable settings
- **Automated Cron Job** — documented cron schedule for daily 02:00 UTC backups
  ```cron
  0 2 * * * /workspaces/PERO-J/scripts/backup.sh >> /var/log/backup.log 2>&1
  ```
- **Restore Procedure** — step-by-step instructions for restoring from a backup dump, including restore to a new database for verification
- **Cloud Deployment Backups** — table documenting how to enable automated backups on AWS RDS, Google Cloud SQL, Supabase, and Neon, plus recommendation for weekly `pg_dump` exports to object storage (S3, GCS) as offsite copies

### 3. Updated: `ROADMAP.md` — Tranche 3 Deliverable 3.2

Updated deliverable 3.2 from:
> PostgreSQL hosted with daily backups

To:
> PostgreSQL hosted with daily automated backups via `scripts/backup.sh` and cron

This reflects that the backup strategy is now implemented and documented.

## closes #106