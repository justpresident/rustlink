use crate::app::App;
use crate::model::ToolState;

pub fn handle_command(app: &mut App) {
    let input = app.input.trim().to_string();
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => app.logs.push(
            "Commands: connect <ip>, ls, scp <file>, crack, run <tool>, disconnect, clear, exit"
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
                if *tool == "PasswordBreaker" {
                    app.active_tool = ToolState::Running {
                        progress: 0.0,
                        target_ip: target.clone(),
                    };
                    app.logs
                        .push(format!("Running PasswordBreaker on {}...", target));
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
        "crack" => {
            if let Some(target) = &app.target_ip {
                app.active_tool = ToolState::Running {
                    progress: 0.0,
                    target_ip: target.clone(),
                };
                app.logs.push(format!("Cracking {}...", target));
            }
        }
        "disconnect" => app.reset_connection(),
        "exit" => app.should_quit = true,
        _ => app.logs.push(format!("Unknown command: {}", parts[0])),
    }
    app.input.clear();
}
