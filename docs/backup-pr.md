## Description

Add database backup automation and documentation for PostgreSQL.

The ROADMAP (Tranche 3, item 3.2) specifies "PostgreSQL hosted with daily
backups" but there was no backup strategy implemented or documented.
Without backups, all decoded event history and registered ABI metadata is
lost on database failure.

### Changes

- **`scripts/backup.sh`** — pg_dump-based backup script with:
  - Configurable `DATABASE_URL`, `BACKUP_DIR`, and `RETENTION_DAYS`
  - Automatic compression (gzip)
  - Retention-based cleanup of old backups (default: 7 days)
- **`docs/backup.md`** — comprehensive documentation covering:
  - Script usage and environment variables
  - Cron job setup for daily automated backups (`0 2 * * *`)
  - Restore procedure (local and cross-database)
  - Cloud deployment backup configuration:
    - AWS RDS (automated backups + manual snapshots via `aws rds`)
    - Google Cloud SQL (automated backups + manual `gcloud sql backups`)
    - Azure Database for PostgreSQL
  - Backup testing and monitoring guidance
- **`Makefile`** — added `backup` and `backup-restore` targets for convenience
- **`.env.example`** — added `BACKUP_DIR` and `RETENTION_DAYS` variables
- **`.gitignore`** — exclude generated `backups/` directory from version control

### Testing

- The backup script was made executable and verified with shellcheck-compatible
  syntax
- Restore procedure was documented with copy-paste commands
- To test manually:
  ```bash
  ./scripts/backup.sh
  gunzip -c backups/soroban_explorer_*.sql.gz | psql "$DATABASE_URL"
  ```

closes #106
