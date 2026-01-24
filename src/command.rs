use crate::app::App;
use crate::model::{ToolState, ToolType};

pub fn handle_command(app: &mut App) {
    let input = app.input.trim().to_string();
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => app.logs.push(
            "Commands: connect <ip>, ls, scp <file>, run <tool>, inbox, read <id>, delete <id>, disconnect, clear, exit"
                .into(),
        ),
        "clear" => app.logs.clear(),
        "connect" => {
            if let Some(&ip) = parts.get(1) {
                if app.servers.contains_key(ip) {
                    app.connection_path.push(ip.to_string());
                    app.target_ip = Some(ip.to_string());
                    app.is_tracing = true; // Starting a connection starts the trace!
                    app.logs.push(format!("Connected to {}", ip));
                } else {
                    app.logs.push(format!("Error: Unknown IP {}", ip));
                }
            }
        }
        "ls" => {
            if let Some(target) = &app.target_ip {
                let server = &app.servers[target];
                if server.is_locked {
                    app.logs.push("Access Denied: Server Locked".into());
                } else {
                    let files: Vec<String> =
                        server.fs.files.iter().map(|f| f.name.clone()).collect();
                    app.logs.push(format!("Files: {}", files.join(", ")));
                }
            }
        }
        "run" => {
            if let (Some(tool), Some(target)) = (parts.get(1), &app.target_ip) {
                match *tool {
                    "PasswordBreaker" => {
                        app.active_tool = ToolState::Running {
                            progress: 0.0,
                            target_ip: target.clone(),
                            tool_type: ToolType::PasswordBreaker,
                        };
                        app.logs
                            .push(format!("Running PasswordBreaker on {}...", target));
                    }
                    "FirewallBuster" => {
                        let server = &app.servers[target];
                        if server.firewall.is_some() {
                            app.active_tool = ToolState::Running {
                                progress: 0.0,
                                target_ip: target.clone(),
                                tool_type: ToolType::FirewallBuster,
                            };
                            app.logs
                                .push(format!("Running FirewallBuster on {}...", target));
                        } else {
                            app.logs.push("No firewall detected on target.".into());
                        }
                    }
                    _ => app.logs.push(format!("Unknown tool: {}", tool)),
                }
            }
        }
        "scp" => {
            if let (Some(target), Some(&fname)) = (app.target_ip.clone(), parts.get(1)) {
                let server = &app.servers[&target];
                if server.is_locked {
                    app.logs.push("Access Denied: Server Locked".into());
                } else if let Some(f) = server.fs.files.iter().find(|f| f.name == fname) {
                    app.local_files.push(f.clone());
                    app.logs
                        .push(format!("File '{}' downloaded successfully.", fname));
                    app.check_missions(fname);
                } else {
                    app.logs.push(format!("File '{}' not found.", fname));
                }
            }
        }
        "inbox" => {
            if app.inbox.is_empty() {
                app.logs.push("Inbox is empty.".into());
            } else {
                app.logs.push("--- INBOX ---".into());
                for mail in &app.inbox {
                    let status = if mail.is_read { "[READ]" } else { "[NEW]" };
                    app.logs.push(format!(
                        "ID: {} {} From: {} Subject: {}",
                        mail.id, status, mail.sender, mail.subject
                    ));
                }
                app.logs.push("-------------".into());
            }
        }
        "read" => {
            if let Some(id_str) = parts.get(1) {
                if let Ok(id) = id_str.parse::<u32>() {
                    if let Some(mail) = app.inbox.iter_mut().find(|m| m.id == id) {
                        app.logs.push(format!("--- MAIL ID: {} ---", mail.id));
                        app.logs.push(format!("From: {}", mail.sender));
                        app.logs.push(format!("Subject: {}", mail.subject));
                        app.logs.push("".into());
                        app.logs.push(mail.body.clone());
                        app.logs.push("-------------------".into());
                        mail.is_read = true;
                    } else {
                        app.logs.push(format!("Error: Mail with ID {} not found.", id));
                    }
                } else {
                    app.logs.push("Error: Invalid mail ID.".into());
                }
            } else {
                app.logs.push("Usage: read <mail_id>".into());
            }
        }
        "delete" => {
            if let Some(id_str) = parts.get(1) {
                if let Ok(id) = id_str.parse::<u32>() {
                    let initial_len = app.inbox.len();
                    app.inbox.retain(|m| m.id != id);
                    if app.inbox.len() < initial_len {
                        app.logs.push(format!("Mail with ID {} deleted.", id));
                    } else {
                        app.logs.push(format!("Error: Mail with ID {} not found.", id));
                    }
                } else {
                    app.logs.push("Error: Invalid mail ID.".into());
                }
            } else {
                app.logs.push("Usage: delete <mail_id>".into());
            }
        }
        "disconnect" => app.reset_connection(),
        "exit" => app.should_quit = true,
        _ => app.logs.push(format!("Unknown command: {}", parts[0])),
    }
    app.input.clear();
}

