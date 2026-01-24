use super::{Command, CommandResult};
use crate::{app::App, commands::CommandRegistry};

pub struct InboxCommand;

impl Command for InboxCommand {
    fn name(&self) -> &'static str {
        "inbox"
    }

    fn aliases(&self) -> &[&'static str] {
        &["mail", "messages"]
    }

    fn description(&self) -> &'static str {
        "Show mail inbox"
    }

    fn execute(&self, app: &mut App, _args: &[&str], _registry: &CommandRegistry) -> CommandResult {
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
        CommandResult::Ok
    }
}

pub struct ReadMailCommand;

impl Command for ReadMailCommand {
    fn name(&self) -> &'static str {
        "read"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn description(&self) -> &'static str {
        "Read a mail message"
    }

    fn usage(&self) -> &'static str {
        "read <id>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        let Some(&id_str) = args.first() else {
            app.logs.push("Usage: read <mail_id>".into());
            return CommandResult::Ok;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.logs.push("Error: Invalid mail ID.".into());
            return CommandResult::Ok;
        };

        if let Some(mail) = app.inbox.iter_mut().find(|m| m.id == id) {
            app.logs.push(format!("--- MAIL ID: {} ---", mail.id));
            app.logs.push(format!("From: {}", mail.sender));
            app.logs.push(format!("Subject: {}", mail.subject));
            app.logs.push("".into());
            // Split body into lines for better display
            for line in mail.body.lines() {
                app.logs.push(line.to_string());
            }
            app.logs.push("-------------------".into());
            mail.is_read = true;
        } else {
            app.logs
                .push(format!("Error: Mail with ID {} not found.", id));
        }

        CommandResult::Ok
    }

    fn completions(&self, app: &App, _arg_index: usize, prefix: &str) -> Vec<String> {
        app.inbox
            .iter()
            .map(|m| m.id.to_string())
            .filter(|id| id.starts_with(prefix))
            .collect()
    }
}

pub struct DeleteMailCommand;

impl Command for DeleteMailCommand {
    fn name(&self) -> &'static str {
        "delete"
    }

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn description(&self) -> &'static str {
        "Delete a mail message"
    }

    fn usage(&self) -> &'static str {
        "delete <id>"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        let Some(&id_str) = args.first() else {
            app.logs.push("Usage: delete <mail_id>".into());
            return CommandResult::Ok;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.logs.push("Error: Invalid mail ID.".into());
            return CommandResult::Ok;
        };

        let initial_len = app.inbox.len();
        app.inbox.retain(|m| m.id != id);

        if app.inbox.len() < initial_len {
            app.logs.push(format!("Mail {} deleted.", id));
        } else {
            app.logs
                .push(format!("Error: Mail with ID {} not found.", id));
        }

        CommandResult::Ok
    }

    fn completions(&self, app: &App, _arg_index: usize, prefix: &str) -> Vec<String> {
        app.inbox
            .iter()
            .map(|m| m.id.to_string())
            .filter(|id| id.starts_with(prefix))
            .collect()
    }
}
