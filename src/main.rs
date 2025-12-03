use std::io;
use std::time::Duration;
use ratatui::{
    backend::CrosstermBackend,
    widgets::Paragraph,
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;

fn pos(row: usize, col: usize, width: usize) -> usize {
    row * width + col
}

/// Wrapping position - handles negative indices for toroidal grid
fn pos_wrap(row: isize, col: isize, width: usize, height: usize) -> usize {
    let r = row.rem_euclid(height as isize) as usize;
    let c = col.rem_euclid(width as isize) as usize;
    pos(r, c, width)
}

/// Apply Game of Life rules at a given position
/// Returns true if cell should be alive in next generation
fn cell_next_state(grid: &[bool], row: isize, col: isize, width: usize, height: usize) -> bool {
    // Count live neighbors (8 surrounding cells)
    let mut neighbors = 0;
    for dr in -1..=1 {
        for dc in -1..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            if grid[pos_wrap(row + dr, col + dc, width, height)] {
                neighbors += 1;
            }
        }
    }

    let alive = grid[pos_wrap(row, col, width, height)];

    // Game of Life rules:
    // 1. Live cell with 2 or 3 neighbors survives
    // 2. Dead cell with exactly 3 neighbors becomes alive
    // 3. All other cells die or stay dead
    match (alive, neighbors) {
        (true, 2) | (true, 3) => true,
        (false, 3) => true,
        _ => false,
    }
}

/// Create next generation grid from current grid
fn next_generation(grid: &[bool], width: usize, height: usize) -> Vec<bool> {
    (0..height)
        .flat_map(|row| {
            (0..width).map(move |col| cell_next_state(grid, row as isize, col as isize, width, height))
        })
        .collect()
}

fn render_grid(frame: &mut Frame, grid: &[bool], width: usize, height: usize) {
    let area = frame.area();
    
    let mut content = String::new();
    for row in 0..area.height as usize {
        for col in 0..area.width as usize {
            if row < height && col < width {
                if grid[pos(row, col, width)] {
                    content.push('█'); // Full block for true
                } else {
                    content.push(' '); // Space for false
                }
            } else {
                content.push(' ');
            }
        }
        if row < area.height as usize - 1 {
            content.push('\n');
        }
    }
    
    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, area);
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Get terminal size and create randomized flat array of bools
    let size = terminal.size()?;
    let width = size.width as usize;
    let height = size.height as usize;
    let mut rng = rand::rng();
    let mut grid: Vec<bool> = (0..width * height).map(|_| rng.random_bool(0.5)).collect();

    // Main loop
    let frame_duration = Duration::from_secs_f64(1.0 / 30.0);
    
    loop {
        terminal.draw(|frame| {
            render_grid(frame, &grid, width, height);
        })?;

        // Poll for key press with timeout
        if event::poll(frame_duration)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        // Advance to next generation
        grid = next_generation(&grid, width, height);
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
