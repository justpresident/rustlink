use super::Tool;
use crate::app::App;

pub struct FirewallBuster;

impl Tool for FirewallBuster {
    fn name(&self) -> &'static str {
        "FirewallBuster"
    }

    fn description(&self) -> &'static str {
        "Disables server firewalls"
    }

    fn can_run(&self, app: &App, target_ip: &str) -> Result<(), String> {
        let Some(server) = app.servers.get(target_ip) else {
            return Err("Target server not found.".into());
        };

        match &server.firewall {
            None => Err("No firewall detected on target.".into()),
            Some(fw) if !fw.is_active => Err("Firewall is already disabled.".into()),
            Some(_) => Ok(()),
        }
    }

    fn on_tick(&self, current_progress: f64) -> f64 {
        // FirewallBuster is slower than PasswordBreaker
        (current_progress + 1.8).min(100.0)
    }

    fn on_complete(&self, app: &mut App, target_ip: &str) {
        if let Some(server) = app.servers.get_mut(target_ip) {
            if let Some(firewall) = &mut server.firewall {
                firewall.is_active = false;
                app.logs.push(format!("SUCCESS: Firewall disabled on {}", target_ip));
            }
        }
    }
}
