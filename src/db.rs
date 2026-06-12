use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use crate::models::{Bank, SearchResponse};

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_pool(db_path: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder().max_size(10).build(manager)?;

    let conn = pool.get()?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS banks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL,
            name TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_banks_code ON banks(code);
        CREATE INDEX IF NOT EXISTS idx_banks_name ON banks(name);",
    )?;

    // Migrate: remove duplicates on code, then add unique index for upsert
    conn.execute(
        "DELETE FROM banks WHERE id NOT IN (SELECT MIN(id) FROM banks GROUP BY code)",
        [],
    )?;
    conn.execute_batch("CREATE UNIQUE INDEX IF NOT EXISTS idx_banks_code_unique ON banks(code);")?;

    Ok(pool)
}

pub fn search_banks(
    pool: &DbPool,
    query: &str,
) -> Result<SearchResponse, Box<dyn std::error::Error>> {
    let conn = pool.get()?;
    let pattern = format!("%{}%", query);

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM banks WHERE code LIKE ?1 OR name LIKE ?1",
        params![pattern],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, code, name FROM banks WHERE code LIKE ?1 OR name LIKE ?1 ORDER BY id LIMIT 50",
    )?;

    let banks = stmt
        .query_map(params![pattern], |row| {
            Ok(Bank {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SearchResponse {
        results: banks,
        total: count as usize,
    })
}

/// Upsert: if code exists, update name; otherwise insert.
pub fn insert_banks(
    pool: &DbPool,
    banks: &[(String, String)],
) -> Result<usize, Box<dyn std::error::Error>> {
    let conn = pool.get()?;
    let mut count = 0;
    for (code, name) in banks {
        conn.execute(
            "INSERT INTO banks (code, name) VALUES (?1, ?2)
             ON CONFLICT(code) DO UPDATE SET name = excluded.name",
            params![code, name],
        )?;
        count += 1;
    }
    Ok(count)
}

pub fn update_bank(
    pool: &DbPool,
    id: i64,
    code: &str,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let conn = pool.get()?;
    let rows = conn.execute(
        "UPDATE banks SET code = ?1, name = ?2 WHERE id = ?3",
        params![code, name, id],
    )?;
    Ok(rows > 0)
}

pub fn delete_bank(pool: &DbPool, id: i64) -> Result<bool, Box<dyn std::error::Error>> {
    let conn = pool.get()?;
    let rows = conn.execute("DELETE FROM banks WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

pub fn clear_banks(pool: &DbPool) -> Result<usize, Box<dyn std::error::Error>> {
    let conn = pool.get()?;
    let count = conn.execute("DELETE FROM banks", [])?;
    Ok(count)
}
