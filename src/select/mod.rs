use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
#[cfg_attr(debug_assertions, allow(unused_imports))]
use crossterm::terminal::enable_raw_mode;
use crossterm::{execute, terminal};
use std::io::stdout;
pub fn select_multiple(options: Vec<String>) -> Vec<usize> {
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        MoveTo(0, 0)
    )
    .unwrap();
    #[cfg(unix)]
    {
        enable_raw_mode().unwrap();
    }
    let mut cursor_position = 0;
    let mut selected_indices = Vec::new();

    loop {
        #[cfg(windows)]
        {
            let _ = read();
        }
        #[cfg(unix)]
        {
            println!("当前没有需要更新的项目，请使用上下键选择、空格键选中、回车键确认部署:\r");
            println!("---------------------------------------------------------------------\r");
        }
        #[cfg(windows)]
        {
            println!("当前没有需要更新的项目，请使用上下键选择、空格键选中、回车键确认部署:");
            println!("---------------------------------------------------------------------");
        }
        for (index, option) in options.iter().enumerate() {
            let prefix = if selected_indices.contains(&index) {
                "[*]"
            } else {
                "[ ]"
            };
            // 高亮当前行，换行显示
            if index == cursor_position {
                execute!(
                    stdout(),
                    SetForegroundColor(Color::Red),
                    Print(format!("{} {}", prefix, option)),
                    ResetColor,
                    Print("\r\n")
                )
                .unwrap();
            } else {
                execute!(
                    stdout(),
                    Print(format!("{} {}", prefix, option)),
                    Print("\r\n")
                )
                .unwrap();
            }
        }

        if let Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) = read()
        {
            match (code, modifiers) {
                (KeyCode::Up, _) => {
                    if cursor_position > 0 {
                        cursor_position -= 1;
                    }
                }
                (KeyCode::Down, _) => {
                    if cursor_position < options.len() - 1 {
                        cursor_position += 1;
                    }
                }
                (KeyCode::Char(' '), _) => {
                    if selected_indices.contains(&cursor_position) {
                        selected_indices.retain(|&x| x != cursor_position);
                    } else {
                        selected_indices.push(cursor_position);
                    }
                }
                (KeyCode::Enter, _) => {
                    return selected_indices;
                }
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    #[cfg(unix)]
                    {
                        std::process::exit(0);
                    }
                }
                _ => {}
            }

            execute!(
                stdout(),
                terminal::Clear(terminal::ClearType::All),
                MoveTo(0, 0)
            )
            .unwrap();
        }
    }
}
