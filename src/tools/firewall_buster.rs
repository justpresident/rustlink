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

    fn on_tick(&self, app: &App, target_ip: &str, current_progress: f64) -> f64 {
        // Speed depends on firewall strength (higher strength = slower)
        let strength = app
            .servers
            .get(target_ip)
            .and_then(|s| s.firewall.as_ref())
            .map(|fw| fw.strength)
            .unwrap_or(50);

        // Base speed of 3.0, reduced by strength (strength 100 = 0.5, strength 0 = 3.0)
        let speed = 3.0 - (strength as f64 / 100.0 * 2.5);
        (current_progress + speed).min(100.0)
    }

    fn on_complete(&self, app: &mut App, target_ip: &str) {
        if let Some(server) = app.servers.get_mut(target_ip)
            && let Some(firewall) = &mut server.firewall
        {
            firewall.is_active = false;
            app.log(format!("SUCCESS: Firewall disabled on {}", target_ip));
        }
    }
}
