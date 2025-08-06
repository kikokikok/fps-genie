-- Test data for CS2 demo analysis system
-- This script creates sample data for development and testing

-- Insert sample tournaments
INSERT INTO tournaments (name, start_date, end_date, prize_pool, tier) VALUES
('ESL Pro League S18', '2023-08-01', '2023-08-20', 750000, 'S'),
('BLAST Premier Fall Groups', '2023-09-15', '2023-09-25', 425000, 'A'),
('IEM Katowice 2024', '2024-02-01', '2024-02-11', 1000000, 'S');

-- Insert sample teams
INSERT INTO teams (name, region, current_ranking) VALUES
('NAVI', 'Europe', 1),
('FaZe Clan', 'Europe', 2),
('Vitality', 'Europe', 3),
('Astralis', 'Europe', 5),
('G2 Esports', 'Europe', 4);

-- Insert sample players
INSERT INTO players (steam_id, nickname, real_name, team_id, role) VALUES
('76561198034202275', 's1mple', 'Oleksandr Kostyliev', 1, 'awper'),
('76561198010511021', 'rain', 'Håvard Nygaard', 2, 'rifler'),
('76561198004854956', 'ZywOo', 'Mathieu Herbaut', 3, 'awper'),
('76561197987713664', 'device', 'Nicolai Reedtz', 4, 'awper'),
('76561197979669175', 'NiKo', 'Nikola Kovač', 5, 'rifler');

-- Insert sample matches
INSERT INTO matches (
    match_id, tournament_id, team1_id, team2_id, map_name,
    match_date, team1_score, team2_score, demo_file_path,
    processing_status, demo_file_size
) VALUES
('navi_vs_faze_dust2_2023', 1, 1, 2, 'de_dust2',
 '2023-08-05 14:30:00', 16, 12, 'test_data/navi_vs_faze_dust2.dem',
 'completed', 128456789),
('vitality_vs_astralis_mirage_2023', 1, 3, 4, 'de_mirage',
 '2023-08-06 16:00:00', 16, 14, 'test_data/vitality_vs_astralis_mirage.dem',
 'completed', 142367891),
('g2_vs_navi_inferno_2023', 2, 5, 1, 'de_inferno',
 '2023-09-18 19:30:00', 13, 16, 'test_data/g2_vs_navi_inferno.dem',
 'pending', 156789234);

-- Insert sample key moments (for testing ML pipeline)
INSERT INTO key_moments (
    match_id, round_number, tick, moment_type, player_steam_id,
    description, significance_score
) VALUES
('navi_vs_faze_dust2_2023', 15, 45600, 'clutch_1v2', '76561198034202275',
 's1mple 1v2 clutch with AWP on A site', 0.95),
('vitality_vs_astralis_mirage_2023', 28, 89200, 'ace', '76561198004854956',
 'ZywOo ace round with deagle and rifle', 0.98),
('navi_vs_faze_dust2_2023', 7, 21800, 'entry_frag', '76561198034202275',
 's1mple AWP pick on long doors', 0.75);

-- Create some sample behavioral vectors (simplified for testing)
INSERT INTO behavioral_vectors (
    vector_id, match_id, player_steam_id, tick, vector_type,
    embedding, metadata
) VALUES
(gen_random_uuid(), 'navi_vs_faze_dust2_2023', '76561198034202275', 45600, 'clutch_moment',
 ARRAY[0.1, 0.8, 0.3, 0.9, 0.2, 0.7, 0.4, 0.6]::float[],
 '{"weapon": "awp", "position": "A_site", "enemies_alive": 2, "health": 78}'::jsonb),
(gen_random_uuid(), 'vitality_vs_astralis_mirage_2023', '76561198004854956', 89200, 'multi_kill',
 ARRAY[0.9, 0.2, 0.8, 0.1, 0.7, 0.3, 0.6, 0.9]::float[],
 '{"weapon": "ak47", "kills_this_round": 5, "position": "B_apps", "health": 100}'::jsonb);

-- Update statistics
ANALYZE;
