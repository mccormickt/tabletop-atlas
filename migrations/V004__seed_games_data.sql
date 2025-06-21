-- Migration to seed games table with BoardGameGeek rankings data
-- This migration loads the top-ranked board games from BoardGameGeek

-- First, create a temporary table to hold the CSV data
CREATE TEMPORARY TABLE temp_bgg_games (
    bgg_id INTEGER,
    name TEXT,
    year_published INTEGER,
    rank INTEGER,
    bayes_average REAL,
    average REAL,
    users_rated INTEGER,
    is_expansion INTEGER,
    abstracts_rank INTEGER,
    cgs_rank INTEGER,
    childrens_games_rank INTEGER,
    family_games_rank INTEGER,
    party_games_rank INTEGER,
    strategy_games_rank INTEGER,
    thematic_rank INTEGER,
    wargames_rank INTEGER
);

-- Note: In a real application, you would use a CSV import mechanism
-- For now, we'll insert some sample data manually to demonstrate the structure
-- In production, you'd want to use a tool like sqlite3's .import command
-- or handle CSV loading in the Rust application code

-- Insert sample top-ranked games from BGG
INSERT INTO games (name, year_published, bgg_id, complexity_rating, min_players, max_players, play_time_minutes, publisher, description) VALUES
('Brass: Birmingham', 2018, 224517, 3.9, 2, 4, 120, 'Roxley', 'An economic strategy game set in Birmingham during the industrial revolution'),
('Pandemic Legacy: Season 1', 2015, 161936, 2.8, 2, 4, 60, 'Z-Man Games', 'A cooperative legacy game about saving the world from diseases'),
('Ark Nova', 2021, 342942, 3.7, 1, 4, 150, 'Feuerland Spiele', 'Plan and design a modern, scientifically managed zoo'),
('Gloomhaven', 2017, 174430, 3.9, 1, 4, 120, 'Cephalofair Games', 'A game of Euro-inspired tactical combat in a persistent world'),
('Twilight Imperium: Fourth Edition', 2017, 233078, 4.2, 3, 6, 480, 'Fantasy Flight Games', 'Build an empire to claim the throne of the galaxy'),
('Dune: Imperium', 2020, 316554, 3.0, 1, 4, 120, 'Dire Wolf', 'A game that finds inspiration in elements and characters from the Dune legacy'),
('Terraforming Mars', 2016, 167791, 3.2, 1, 5, 120, 'FryxGames', 'Compete to transform Mars into a habitable planet'),
('War of the Ring: Second Edition', 2011, 115746, 4.2, 2, 4, 180, 'Ares Games', 'One player takes control of the Free Peoples, the other controls the Shadow Armies'),
('Star Wars: Rebellion', 2016, 187645, 3.7, 2, 4, 240, 'Fantasy Flight Games', 'Strike at the Death Star, or perfect your plans for galactic domination'),
('Wingspan', 2019, 266192, 2.4, 1, 5, 70, 'Stonemaier Games', 'Attract a beautiful and diverse collection of birds to your wildlife preserves');

-- Create an index on bgg_id for the games we just inserted
CREATE INDEX IF NOT EXISTS idx_games_bgg_id_seed ON games(bgg_id) WHERE bgg_id IS NOT NULL;

-- Add a comment about the seeding process
-- Note: This migration provides a foundation of popular games
-- Additional games can be added through the API or additional migrations
-- The CSV file contains thousands of games that could be bulk-loaded
-- using application code or database import tools
