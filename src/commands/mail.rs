use super::{Command, CommandResult};
use crate::{app::App, commands::CommandRegistry};

pub struct MailCommand;

impl MailCommand {
    fn show_inbox(&self, app: &mut App) {
        if app.inbox.is_empty() {
            app.log("Inbox is empty.");
        } else {
            // Collect mail info first to avoid borrow issues
            let mail_lines: Vec<_> = app
                .inbox
                .iter()
                .map(|mail| {
                    let status = if mail.is_read { " " } else { "*" };
                    format!(
                        " {} [{}] From: {} - {}",
                        status, mail.id, mail.sender, mail.subject
                    )
                })
                .collect();
            let unread = app.inbox.iter().filter(|m| !m.is_read).count();

            app.log("--- INBOX ---");
            for line in mail_lines {
                app.log(line);
            }
            app.log("-------------");
            if unread > 0 {
                app.log(format!("{} unread message(s)", unread));
            }
        }
    }

    fn read_mail(&self, app: &mut App, args: &[&str]) {
        let Some(&id_str) = args.first() else {
            app.log("Usage: mail read <mail_id>");
            return;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.log("Error: Invalid mail ID.");
            return;
        };

        // Find and extract mail content first
        let mail_content = app.inbox.iter().find(|m| m.id == id).map(|mail| {
            (
                mail.id,
                mail.sender.clone(),
                mail.subject.clone(),
                mail.body.clone(),
            )
        });

        if let Some((mail_id, sender, subject, body)) = mail_content {
            app.log(format!("--- MAIL ID: {} ---", mail_id));
            app.log(format!("From: {}", sender));
            app.log(format!("Subject: {}", subject));
            app.log(" ");
            for line in body.lines() {
                app.log(line);
            }
            app.log("-------------------");
            // Mark as read after logging
            if let Some(mail) = app.inbox.iter_mut().find(|m| m.id == id) {
                mail.is_read = true;
            }
        } else {
            app.log(format!("Error: Mail with ID {} not found.", id));
        }
    }

    fn delete_mail(&self, app: &mut App, args: &[&str]) {
        let Some(&id_str) = args.first() else {
            app.log("Usage: mail delete <mail_id>");
            return;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.log("Error: Invalid mail ID.");
            return;
        };

        let initial_len = app.inbox.len();
        app.inbox.retain(|m| m.id != id);

        if app.inbox.len() < initial_len {
            app.log(format!("Mail {} deleted.", id));
        } else {
            app.log(format!("Error: Mail with ID {} not found.", id));
        }
    }
}

impl Command for MailCommand {
    fn name(&self) -> &'static str {
        "mail"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
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
                app.log(format!("Unknown subcommand: {}", subcmd));
                app.log("Usage: mail [read|delete] [id]");
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
