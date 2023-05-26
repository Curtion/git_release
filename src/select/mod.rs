use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::style::{SetForegroundColor, Color, ResetColor, Print};
use crossterm::{execute, terminal};
use std::io::stdout;

pub fn select_multiple(options: Vec<String>) -> Vec<usize> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All), MoveTo(0, 0)).unwrap();
    let mut cursor_position = 0;
    let mut selected_indices = Vec::new();

    loop {
        let _ = read();
        println!("当前没有需要更新的项目，请使用上下键选择、空格键选中、回车键确认部署:");
        println!("---------------------------------------------------------------------");
        for (index, option) in options.iter().enumerate() {
            let prefix = if selected_indices.contains(&index) {
                "[*]"
            } else {
                "[ ]"
            };
            // 高亮当前行，换行显示
            if index == cursor_position {
                execute!(stdout(), SetForegroundColor(Color::Red), Print(format!("{} {}", prefix, option)), ResetColor, Print("\n")).unwrap();
            } else {
                execute!(stdout(), Print(format!("{} {}", prefix, option)), Print("\n")).unwrap();
            }
        }
        
        if let Ok(Event::Key(KeyEvent { code, .. })) = read() {
            match code {
                KeyCode::Up => {
                    if cursor_position > 0 {
                        cursor_position -= 1;
                    }
                }
                KeyCode::Down => {
                    if cursor_position < options.len() - 1 {
                        cursor_position += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    if selected_indices.contains(&cursor_position) {
                        selected_indices.retain(|&x| x != cursor_position);
                    } else {
                        selected_indices.push(cursor_position);
                    }
                }
                KeyCode::Enter => {
                    return selected_indices;
                }
                _ => {}
            }

            execute!(stdout(), terminal::Clear(terminal::ClearType::All), MoveTo(0, 0)).unwrap();
        }
    }
}