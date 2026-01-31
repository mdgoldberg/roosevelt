      SELECT
          p.name,
          COUNT(*) as games_played,
          ROUND(100.0 * SUM(CASE WHEN gr.finishing_role = 'President' THEN 1 ELSE 0 END) / COUNT(*), 2) as president_pct,
          ROUND(100.0 * SUM(CASE WHEN gr.finishing_role = 'VicePresident' THEN 1 ELSE 0 END) / COUNT(*), 2) as vice_president_pct,
          ROUND(100.0 * SUM(CASE WHEN gr.finishing_role = 'Secretary' THEN 1 ELSE 0 END) / COUNT(*), 2) as secretary_pct,
          ROUND(100.0 * SUM(CASE WHEN gr.finishing_role = 'ViceAsshole' THEN 1 ELSE 0 END) / COUNT(*), 2) as vice_asshole_pct,
          ROUND(100.0 * SUM(CASE WHEN gr.finishing_role = 'Asshole' THEN 1 ELSE 0 END) / COUNT(*), 2) as asshole_pct,
          SUM(CASE WHEN gr.finishing_role = 'President' THEN 1 ELSE 0 END) as president_count,
          SUM(CASE WHEN gr.finishing_role = 'VicePresident' THEN 1 ELSE 0 END) as vice_president_count,
          SUM(CASE WHEN gr.finishing_role = 'Secretary' THEN 1 ELSE 0 END) as secretary_count,
          SUM(CASE WHEN gr.finishing_role = 'ViceAsshole' THEN 1 ELSE 0 END) as vice_asshole_count,
          SUM(CASE WHEN gr.finishing_role = 'Asshole' THEN 1 ELSE 0 END) as asshole_count,
          ROUND(AVG(gr.finishing_place), 2) as avg_finishing_place,
          SUM(CASE WHEN gr.finishing_place = 1 THEN 1 ELSE 0 END) as wins,
          ROUND(100.0 * SUM(CASE WHEN gr.finishing_place = 1 THEN 1 ELSE 0 END) / COUNT(*), 2) as win_rate
     FROM players p
     JOIN game_results gr ON p.id = gr.player_id
     GROUP BY p.id
     ORDER BY win_rate DESC, avg_finishing_place ASC;
