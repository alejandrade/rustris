use crate::rustris_paths;
use diesel::prelude::*;
use std::path::PathBuf;

// Diesel table schema for Lutris pga.db
mod schema {
    diesel::table! {
        games (id) {
            id -> Integer,
            name -> Nullable<Text>,
            sortname -> Nullable<Text>,
            slug -> Nullable<Text>,
            installer_slug -> Nullable<Text>,
            parent_slug -> Nullable<Text>,
            platform -> Nullable<Text>,
            runner -> Nullable<Text>,
            executable -> Nullable<Text>,
            directory -> Nullable<Text>,
            updated -> Nullable<Text>,
            lastplayed -> Nullable<Integer>,
            installed -> Nullable<Integer>,
            installed_at -> Nullable<Integer>,
            year -> Nullable<Integer>,
            configpath -> Nullable<Text>,
            has_custom_banner -> Nullable<Integer>,
            has_custom_icon -> Nullable<Integer>,
            has_custom_coverart_big -> Nullable<Integer>,
            playtime -> Nullable<Float>,
            service -> Nullable<Text>,
            service_id -> Nullable<Text>,
            discord_id -> Nullable<Text>,
        }
    }
}

// Model for a game from the database
#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::games)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LutrisDbGame {
    pub id: i32,
    pub name: Option<String>,
    pub sortname: Option<String>,
    pub slug: Option<String>,
    pub installer_slug: Option<String>,
    pub parent_slug: Option<String>,
    pub platform: Option<String>,
    pub runner: Option<String>,
    pub executable: Option<String>,
    pub directory: Option<String>,
    pub updated: Option<String>,
    pub lastplayed: Option<i32>,
    pub installed: Option<i32>,
    pub installed_at: Option<i32>,
    pub year: Option<i32>,
    pub configpath: Option<String>,
    pub has_custom_banner: Option<i32>,
    pub has_custom_icon: Option<i32>,
    pub has_custom_coverart_big: Option<i32>,
    pub playtime: Option<f32>,
    pub service: Option<String>,
    pub service_id: Option<String>,
    pub discord_id: Option<String>,
}

/// Service for querying Lutris's pga.db database
pub struct LutrisDatabase {
    db_path: PathBuf,
}

impl LutrisDatabase {
    /// Create a new Lutris database service
    pub fn new() -> Result<Self, String> {
        let db_path = rustris_paths::lutris_database()
            .ok_or("Could not get Lutris database path")?;

        if !db_path.exists() {
            return Err(format!("Lutris database not found at {:?}", db_path));
        }

        Ok(Self { db_path })
    }

    /// Establish a connection to the database
    fn connect(&self) -> Result<SqliteConnection, String> {
        SqliteConnection::establish(self.db_path.to_str().unwrap())
            .map_err(|e| format!("Failed to connect to Lutris database: {}", e))
    }

    /// Get the config path for a game by slug
    pub fn get_configpath(&self, game_slug: &str) -> Result<String, String> {
        use schema::games::dsl::*;

        let mut conn = self.connect()?;

        let game: LutrisDbGame = games
            .filter(slug.eq(game_slug))
            .select(LutrisDbGame::as_select())
            .first(&mut conn)
            .map_err(|e| format!("Game '{}' not found in database: {}", game_slug, e))?;

        game.configpath
            .ok_or_else(|| format!("Game '{}' has no config path in database", game_slug))
    }

    /// Get a game by slug
    pub fn get_game_by_slug(&self, game_slug: &str) -> Result<LutrisDbGame, String> {
        use schema::games::dsl::*;

        let mut conn = self.connect()?;

        games
            .filter(slug.eq(game_slug))
            .select(LutrisDbGame::as_select())
            .first(&mut conn)
            .map_err(|e| format!("Game '{}' not found in database: {}", game_slug, e))
    }

    /// Get all installed games
    pub fn get_installed_games(&self) -> Result<Vec<LutrisDbGame>, String> {
        use schema::games::dsl::*;

        let mut conn = self.connect()?;

        games
            .filter(installed.eq(1))
            .select(LutrisDbGame::as_select())
            .load(&mut conn)
            .map_err(|e| format!("Failed to query installed games: {}", e))
    }
}
