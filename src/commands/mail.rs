use super::{Command, CommandResult};
use crate::{app::App, commands::CommandRegistry};

pub struct MailCommand;

impl MailCommand {
    fn show_inbox(app: &mut App) {
        if app.player.inbox.is_empty() {
            app.log("Inbox is empty.");
        } else {
            let mail_lines: Vec<_> = app
                .player
                .inbox
                .iter()
                .map(|mail| {
                    let status = if mail.is_read { " " } else { "*" };
                    let job_marker = if mail.mission_id.is_some() {
                        "[JOB]"
                    } else {
                        ""
                    };
                    format!(
                        " {} [{}] {} From: {} - {}",
                        status, mail.id, job_marker, mail.sender, mail.subject
                    )
                })
                .collect();
            let unread = app.player.unread_mail_count();

            app.log("--- INBOX ---");
            for line in mail_lines {
                app.log(line);
            }
            app.log("-------------");
            if unread > 0 {
                app.log(format!("{unread} unread message(s)"));
            }
        }
    }

    fn read_mail(app: &mut App, args: &[&str]) {
        let Some(&id_str) = args.first() else {
            app.log("Usage: mail read <mail_id>");
            return;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.log("Error: Invalid mail ID.");
            return;
        };

        let mail_content = app.player.inbox.iter().find(|m| m.id == id).map(|mail| {
            (
                mail.id,
                mail.sender.clone(),
                mail.subject.clone(),
                mail.body.clone(),
                mail.mission_id,
            )
        });

        if let Some((mail_id, sender, subject, body, mission_id)) = mail_content {
            app.log(format!("--- MAIL ID: {mail_id} ---"));
            app.log(format!("From: {sender}"));
            app.log(format!("Subject: {subject}"));
            app.log(" ");
            for line in body.lines() {
                app.log(line);
            }
            if mission_id.is_some() {
                app.log(" ");
                app.log(format!("Use 'mail accept {mail_id}' to accept this job."));
            }
            app.log("-------------------");
            if let Some(mail) = app.player.inbox.iter_mut().find(|m| m.id == id) {
                mail.is_read = true;
            }
        } else {
            app.log(format!("Error: Mail with ID {id} not found."));
        }
    }

    fn delete_mail(app: &mut App, args: &[&str]) {
        let Some(&id_str) = args.first() else {
            app.log("Usage: mail delete <mail_id>");
            return;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.log("Error: Invalid mail ID.");
            return;
        };

        let initial_len = app.player.inbox.len();
        app.player.inbox.retain(|m| m.id != id);

        if app.player.inbox.len() < initial_len {
            app.log(format!("Mail {id} deleted."));
        } else {
            app.log(format!("Error: Mail with ID {id} not found."));
        }
    }

    fn accept_mission(app: &mut App, args: &[&str]) {
        let Some(&id_str) = args.first() else {
            app.log("Usage: mail accept <mail_id>");
            return;
        };

        let Ok(id) = id_str.parse::<u32>() else {
            app.log("Error: Invalid mail ID.");
            return;
        };

        match app.accept_mission_from_mail(id) {
            Ok(()) => {}
            Err(msg) => {
                app.log(format!("Error: {msg}"));
            }
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
        "mail [read|delete|accept] [id]"
    }

    fn execute(&self, app: &mut App, args: &[&str], _registry: &CommandRegistry) -> CommandResult {
        match args.first().copied() {
            None => Self::show_inbox(app),
            Some("read") => Self::read_mail(app, &args[1..]),
            Some("delete") => Self::delete_mail(app, &args[1..]),
            Some("accept") => Self::accept_mission(app, &args[1..]),
            Some(subcmd) => {
                app.log(format!("Unknown subcommand: {subcmd}"));
                app.log("Usage: mail [read|delete|accept] [id]");
            }
        }
        CommandResult::Ok
    }

    fn completions(&self, app: &App, arg_index: usize, prefix: &str) -> Vec<String> {
        match arg_index {
            0 => ["read", "delete", "accept"]
                .iter()
                .filter(|s| s.starts_with(prefix))
                .map(|s| (*s).to_string())
                .collect(),
            1 => app
                .player
                .inbox
                .iter()
                .map(|m| m.id.to_string())
                .filter(|id| id.starts_with(prefix))
                .collect(),
            _ => Vec::new(),
        }
    }
}
