use crate::{
    ansi::colors::{BLUE, MAGENTA, RESET},
    //ansi::{CLEAR, Csi, EL},
    ui::getcwd_short,
};

const COMMANDS: [&str; 7] = ["choose", "cd", "ls", "exit", "clear", "new", "help"];

const EXIT_ALIASES: [&str; 3] = ["q", "quit", "ex"];
const NEW_ALIASES: [&str; 3] = ["new", "new_session", "make"];
const CLEAR_ALIASES: [&str; 2] = ["c", "cls"];
const HELP_ALIASES: [&str; 2] = ["h", "?"];

//enum Tokens {
//    Directory(Color),
//}
//
pub struct MenuConfig {
    pub prompt: String,
    pub icon: String,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            prompt: "nspm v1.0.0".to_string(),
            icon: ">".to_string(),
        }
    }
}
//
//struct Config {
//    menu_config: MenuConfig,
//    directory_prompt: fn() -> String,
//    default_generated_password_length: u32,
//}
//
//impl Default for Config {
//    fn default() -> Self {
//        Self {
//            menu_config: MenuConfig::default(),
//            directory_prompt: directory_selector_prompt,
//            default_generated_password_length: 14,
//        }
//    }
//}

/// that just looks awful
pub fn all_commands() -> Vec<String> {
    let mut command: Vec<String> = COMMANDS
        .to_vec()
        .iter()
        .map(|s| s.to_owned().to_owned())
        .collect();
    command.append(
        &mut EXIT_ALIASES
            .iter()
            .map(|s| s.to_owned().to_owned())
            .collect(),
    );
    command.append(
        &mut CLEAR_ALIASES
            .iter()
            .map(|s| s.to_owned().to_owned())
            .collect::<Vec<String>>(),
    );
    command.append(
        &mut NEW_ALIASES
            .iter()
            .map(|s| s.to_owned().to_owned())
            .collect::<Vec<String>>(),
    );
    command.append(
        &mut HELP_ALIASES
            .iter()
            .map(|s| s.to_owned().to_owned())
            .collect::<Vec<String>>(),
    );
    command
}

pub fn process_alias(alias: &str) -> &str {
    let alias = alias.trim();
    if COMMANDS.contains(&alias) {
        return alias;
    } else if EXIT_ALIASES.contains(&alias) {
        return "exit";
    } else if CLEAR_ALIASES.contains(&alias) {
        return "clear";
    } else if NEW_ALIASES.contains(&alias) {
        return "new";
    } else if HELP_ALIASES.contains(&alias) {
        return "help";
    }
    ""
}

pub fn directory_selector_prompt() -> String {
    let current_directory = getcwd_short();
    format!("{BLUE}{current_directory}{RESET} {MAGENTA}❯ {RESET}")
}
