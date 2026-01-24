use super::{Command, CommandResult};
use crate::{app::App, commands::CommandRegistry};

pub struct MailCommand;

impl MailCommand {
    fn show_inbox(&self, app: &mut App) {
        if app.inbox.is_empty() {
            app.logs.push("Inbox is empty.".into());
        } else {
            app.logs.push("--- INBOX ---".into());
            for mail in &app.inbox {
                let status = if mail.is_read { " " } else { "*" };
                app.logs.push(format!(
                    " {} [{}] From: {} - {}",
                    status, mail.id, mail.sender, mail.subject
                ));
            }
            app.logs.push("-------------".into());
            let unread = app.inbox.iter().filter(|m| !m.is_read).count();
            if unread > 0 {
                app.logs.push(format!("{} unread message(s)", unread));
            }
        }
    }

    fn read_mail(&self, app: &mut App, args: &[&str]) {
        let Some(&id_str) = args.first() else {
            app.logs.push("Usage: mail read <mail_id>".into());
            return;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.logs.push("Error: Invalid mail ID.".into());
            return;
        };

        if let Some(mail) = app.inbox.iter_mut().find(|m| m.id == id) {
            app.logs.push(format!("--- MAIL ID: {} ---", mail.id));
            app.logs.push(format!("From: {}", mail.sender));
            app.logs.push(format!("Subject: {}", mail.subject));
            app.logs.push("".into());
            for line in mail.body.lines() {
                app.logs.push(line.to_string());
            }
            app.logs.push("-------------------".into());
            mail.is_read = true;
        } else {
            app.logs
                .push(format!("Error: Mail with ID {} not found.", id));
        }
    }

    fn delete_mail(&self, app: &mut App, args: &[&str]) {
        let Some(&id_str) = args.first() else {
            app.logs.push("Usage: mail delete <mail_id>".into());
            return;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.logs.push("Error: Invalid mail ID.".into());
            return;
        };

        let initial_len = app.inbox.len();
        app.inbox.retain(|m| m.id != id);

        if app.inbox.len() < initial_len {
            app.logs.push(format!("Mail {} deleted.", id));
        } else {
            app.logs
                .push(format!("Error: Mail with ID {} not found.", id));
        }
    }
}

impl Command for MailCommand {
    fn name(&self) -> &'static str {
        "mail"
    }

    fn aliases(&self) -> &[&'static str] {
        &["inbox", "messages"]
    }

    fn description(&self) -> &'static str {
        "Manage mail inbox"
    }

    fn usage(&self) -> &'static str {
        "mail [read|delete] [id]"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        match args.first().copied() {
            None => self.show_inbox(app),
            Some("read") => self.read_mail(app, &args[1..]),
            Some("delete") => self.delete_mail(app, &args[1..]),
            Some(subcmd) => {
                app.logs.push(format!("Unknown subcommand: {}", subcmd));
                app.logs.push("Usage: mail [read|delete] [id]".into());
            }
        }
        CommandResult::Ok
    }

    fn completions(&self, app: &App, arg_index: usize, prefix: &str) -> Vec<String> {
        match arg_index {
            0 => {
                // Complete subcommands
                ["read", "delete"]
                    .iter()
                    .filter(|s| s.starts_with(prefix))
                    .map(|s| s.to_string())
                    .collect()
            }
            1 => {
                // Complete mail IDs
                app.inbox
                    .iter()
                    .map(|m| m.id.to_string())
                    .filter(|id| id.starts_with(prefix))
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}
